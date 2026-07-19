package main

import (
	"bufio"
	"context"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

// engine drives the whole copy session: it consumes paths, decides what fits,
// copies with retries, and streams events to a Reporter.
type engine struct {
	opts   options
	report Reporter
	ctx    context.Context

	speedo       speedometer
	sessionStart time.Time
	sessionBytes int64
	lastEmit     time.Time

	freeAtStart uint64
	diskTotal   uint64

	nCopied     int
	copiedBytes int64
	nSkipped    int
	nFailed     int
	stoppedFull bool
}

func newEngine(ctx context.Context, opts options, r Reporter) *engine {
	return &engine{opts: opts, report: r, ctx: ctx}
}

// run reads paths from in until EOF (or a stop condition) and returns the
// session summary. It also emits a final doneMsg for the reporter.
func (e *engine) run(in io.Reader) summary {
	delim := byte('\n')
	if e.opts.null {
		delim = 0
	}
	br := bufio.NewReader(in)

	// Seed the disk figures so the UI has something before the first copy.
	if free, total, err := diskUsage(e.opts.target); err == nil {
		e.freeAtStart, e.diskTotal = free, total
		e.report.Event(diskMsg{free: free, diskTotal: total})
	}

	index := 0
	for {
		tok, rerr := nextPath(br, delim, e.opts.null)
		if tok != "" {
			index++
			if stop := e.process(tok, index); stop {
				break
			}
		}
		if rerr != nil {
			break
		}
	}

	free, total := e.freeAtStart, e.diskTotal
	if f, t, err := diskUsage(e.opts.target); err == nil {
		free, total = f, t
	}
	sum := summary{
		copied:      e.nCopied,
		copiedBytes: e.copiedBytes,
		skipped:     e.nSkipped,
		failed:      e.nFailed,
		elapsed:     e.sessionElapsed(),
		free:        free,
		diskTotal:   total,
		stoppedFull: e.stoppedFull,
	}
	e.report.Event(doneMsg{summary: sum})
	return sum
}

// process handles a single input path. It returns true when the run should
// stop (a non-fitting file without --fill, or a cancellation).
func (e *engine) process(path string, index int) (stop bool) {
	name := filepath.Base(path)

	info, err := os.Stat(path)
	if err != nil {
		e.nFailed++
		e.report.Event(failMsg{name: name, reason: "not found"})
		return false
	}
	if info.IsDir() {
		e.nSkipped++
		e.report.Event(skipMsg{name: name, reason: "directory"})
		return false
	}
	if !info.Mode().IsRegular() {
		e.nSkipped++
		e.report.Event(skipMsg{name: name, reason: "not a regular file"})
		return false
	}
	size := info.Size()
	dest := filepath.Join(e.opts.target, name)

	if !e.opts.force {
		if _, err := os.Stat(dest); err == nil {
			e.nSkipped++
			e.report.Event(skipMsg{name: name, size: size, reason: "already exists"})
			return false
		}
	}

	if free, total, err := diskUsage(e.opts.target); err == nil {
		e.freeAtStart, e.diskTotal = free, total
	}
	need := uint64(size) + e.opts.reserve
	if e.freeAtStart > 0 && need > e.freeAtStart {
		if e.opts.fill {
			e.nSkipped++
			e.report.Event(skipMsg{name: name, size: size, reason: "won't fit"})
			return false
		}
		e.nSkipped++
		e.stoppedFull = true
		e.report.Event(skipMsg{name: name, size: size, reason: "won't fit — stopping", fatal: true})
		return true
	}

	if !e.opts.modest {
		if art := renderThumb(path, thumbCols, thumbRows); art != "" {
			e.report.Event(thumbMsg{name: name, art: art})
		} else {
			e.report.Event(thumbMsg{name: name, art: ""})
		}
	}

	if e.sessionStart.IsZero() {
		e.sessionStart = time.Now()
	}
	e.speedo.reset()
	e.lastEmit = time.Time{}
	fileStart := time.Now()
	freeAtStart := e.freeAtStart

	e.report.Event(fileStartMsg{name: name, path: path, size: size, index: index})

	onProgress := func(copied int64) {
		now := time.Now()
		if !e.lastEmit.IsZero() && now.Sub(e.lastEmit) < 60*time.Millisecond {
			return
		}
		e.lastEmit = now
		inst := e.speedo.sample(now, copied)
		live := freeAtStart
		if uint64(copied) < freeAtStart {
			live = freeAtStart - uint64(copied)
		} else {
			live = 0
		}
		e.report.Event(progressMsg{
			copied:    copied,
			total:     size,
			instSpeed: inst,
			avgSpeed:  e.sessionAvg(now, copied),
			free:      live,
			diskTotal: e.diskTotal,
		})
	}

	var copyErr error
	for attempt := 0; attempt <= e.opts.retries; attempt++ {
		copyErr = copyOne(e.ctx, path, dest, size, e.opts.verify, onProgress)
		if copyErr == nil {
			break
		}
		if e.ctx.Err() != nil {
			return true
		}
		if attempt < e.opts.retries {
			e.report.Event(failMsg{name: name, reason: "retry " + strconv.Itoa(attempt+1) + ": " + copyErr.Error()})
			time.Sleep(250 * time.Millisecond)
		}
	}

	if copyErr != nil {
		if e.ctx.Err() != nil {
			return true
		}
		e.nFailed++
		e.report.Event(failMsg{name: name, reason: copyErr.Error()})
		return false
	}

	e.nCopied++
	e.copiedBytes += size
	e.sessionBytes += size
	dur := time.Since(fileStart)
	e.report.Event(fileDoneMsg{name: name, size: size, dur: dur, verified: e.opts.verify})

	if free, total, err := diskUsage(e.opts.target); err == nil {
		e.freeAtStart, e.diskTotal = free, total
		e.report.Event(diskMsg{free: free, diskTotal: total, avgSpeed: e.sessionAvg(time.Now(), 0)})
	}
	return false
}

func (e *engine) sessionElapsed() time.Duration {
	if e.sessionStart.IsZero() {
		return 0
	}
	return time.Since(e.sessionStart)
}

func (e *engine) sessionAvg(now time.Time, extra int64) float64 {
	if e.sessionStart.IsZero() {
		return 0
	}
	elapsed := now.Sub(e.sessionStart).Seconds()
	if elapsed <= 0 {
		return 0
	}
	return float64(e.sessionBytes+extra) / elapsed
}

// nextPath reads a single token up to delim. It returns the token with the
// delimiter (and, in newline mode, a trailing CR) stripped, along with any
// read error (io.EOF marks the end; the final token before EOF is returned).
func nextPath(br *bufio.Reader, delim byte, null bool) (string, error) {
	chunk, err := br.ReadString(delim)
	tok := chunk
	if len(tok) > 0 && tok[len(tok)-1] == delim {
		tok = tok[:len(tok)-1]
	}
	if !null {
		tok = strings.TrimRight(tok, "\r")
	}
	return tok, err
}
