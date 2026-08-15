package main

import (
	"bufio"
	"context"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"
)

// engine drives the whole copy session: it consumes paths, applies the active
// strategy, decides what fits, copies with retries, and streams events to a
// Reporter.
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

	index       int
	nCopied     int
	copiedBytes int64
	nSkipped    int
	nFiltered   int
	nFailed     int
	stoppedFull bool
}

func newEngine(ctx context.Context, opts options, r Reporter) *engine {
	return &engine{opts: opts, report: r, ctx: ctx}
}

// run consumes paths from in and returns the session summary. It also emits a
// final doneMsg for the reporter.
func (e *engine) run(in io.Reader) summary {
	// Seed the disk figures so the UI has something before the first copy.
	if free, total, err := diskUsage(e.opts.target); err == nil {
		e.freeAtStart, e.diskTotal = free, total
		e.report.Event(diskMsg{free: free, diskTotal: total})
	}

	if e.opts.strategy.spec().sorts {
		e.runSorted(in)
	} else {
		e.runStream(in)
	}

	free, total := e.freeAtStart, e.diskTotal
	if f, t, err := diskUsage(e.opts.target); err == nil {
		free, total = f, t
	}
	sum := summary{
		copied:      e.nCopied,
		copiedBytes: e.copiedBytes,
		skipped:     e.nSkipped,
		filtered:    e.nFiltered,
		failed:      e.nFailed,
		elapsed:     e.sessionElapsed(),
		free:        free,
		diskTotal:   total,
		stoppedFull: e.stoppedFull,
		cancelled:   e.ctx.Err() != nil,
	}
	e.report.Event(doneMsg{summary: sum})
	return sum
}

// runStream judges each path as it arrives and copies it straight away, so a
// non-sorting strategy works on input that is still being produced.
func (e *engine) runStream(in io.Reader) {
	e.eachPath(in, func(path string) bool {
		c := e.vet(path)
		if c == nil {
			return true
		}
		if e.opts.strategy.inspects() {
			e.report.Event(scanMsg{name: c.name})
			prepare(e.ctx, e.opts.strategy, c)
			if e.ctx.Err() != nil {
				return false
			}
		}
		if ok, reason := admit(e.opts.strategy, c); !ok {
			e.filter(c, reason)
			return true
		}
		return !e.copyCandidate(c)
	})
}

// runSorted drains the whole input, inspects every candidate, then copies the
// survivors in the strategy's order.
func (e *engine) runSorted(in io.Reader) {
	var cands []*candidate
	e.eachPath(in, func(path string) bool {
		if c := e.vet(path); c != nil {
			cands = append(cands, c)
		}
		return true
	})

	e.prepareAll(cands)
	if e.ctx.Err() != nil {
		return
	}

	kept := make([]*candidate, 0, len(cands))
	for _, c := range cands {
		if ok, reason := admit(e.opts.strategy, c); ok {
			kept = append(kept, c)
		} else {
			e.filter(c, reason)
		}
	}
	sortCandidates(e.opts.strategy, kept)

	for _, c := range kept {
		if e.copyCandidate(c) {
			return
		}
	}
}

// prepareAll runs the strategy's inspection over every candidate. Probing is
// I/O bound and independent per file, so it runs on a small worker pool while
// the main goroutine — the only one touching the reporter — reports progress.
func (e *engine) prepareAll(cands []*candidate) {
	if len(cands) == 0 || !e.opts.strategy.inspects() {
		return
	}
	const workers = 4

	feed := make(chan int)
	done := make(chan int, len(cands))
	var wg sync.WaitGroup
	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for i := range feed {
				prepare(e.ctx, e.opts.strategy, cands[i])
				done <- i
			}
		}()
	}
	go func() {
		for i := range cands {
			select {
			case feed <- i:
			case <-e.ctx.Done():
				close(feed)
				wg.Wait()
				close(done)
				return
			}
		}
		close(feed)
		wg.Wait()
		close(done)
	}()

	n := 0
	for i := range done {
		n++
		e.report.Event(scanMsg{name: cands[i].name, done: n, total: len(cands)})
	}
}

// eachPath reads paths from in and hands each to fn, stopping early when fn
// returns false.
func (e *engine) eachPath(in io.Reader, fn func(string) bool) {
	delim := byte('\n')
	if e.opts.null {
		delim = 0
	}
	br := bufio.NewReader(in)
	for {
		tok, rerr := nextPath(br, delim, e.opts.null)
		if tok != "" && !fn(tok) {
			return
		}
		if rerr != nil {
			return
		}
	}
}

// vet turns an input path into a candidate, or reports why it can't be one.
// It returns nil for anything that will never be copied — a missing file, a
// directory, or a destination that already exists.
func (e *engine) vet(path string) *candidate {
	name, dest := e.destFor(path)

	info, err := os.Stat(path)
	if err != nil {
		e.nFailed++
		e.report.Event(failMsg{name: name, reason: "not found"})
		return nil
	}
	if info.IsDir() {
		e.nSkipped++
		e.report.Event(skipMsg{name: name, reason: "directory"})
		return nil
	}
	if !info.Mode().IsRegular() {
		e.nSkipped++
		e.report.Event(skipMsg{name: name, reason: "not a regular file"})
		return nil
	}
	// An existing destination is the thing being checked under --verify-only,
	// so only a copying run treats it as a reason to move on.
	if !e.opts.verifyOnly && !e.opts.force && exists(dest) {
		e.nSkipped++
		e.report.Event(skipMsg{name: name, size: info.Size(), reason: "already exists"})
		return nil
	}
	return &candidate{
		path: path,
		name: name,
		dest: dest,
		size: info.Size(),
		mod:  info.ModTime(),
	}
}

// destFor maps an input path to its destination. Relative input keeps its
// structure under the target; absolute input has nowhere sensible to nest, so
// it always lands in the target root, as does everything under --flatten.
func (e *engine) destFor(path string) (name, dest string) {
	base := filepath.Base(path)
	if e.opts.flatten || filepath.IsAbs(path) {
		return base, filepath.Join(e.opts.target, base)
	}
	rel := filepath.Clean(path)
	if rel == "." || rel == ".." || strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
		return base, filepath.Join(e.opts.target, base)
	}
	return rel, filepath.Join(e.opts.target, rel)
}

func (e *engine) filter(c *candidate, reason string) {
	e.nFiltered++
	e.report.Event(filterMsg{name: c.name, reason: reason})
}

// copyCandidate copies one admitted file. It returns true when the run should
// stop: a non-fitting file without --fill, or a cancellation.
func (e *engine) copyCandidate(c *candidate) (stop bool) {
	if e.opts.verifyOnly {
		return e.verifyCandidate(c)
	}
	if free, total, err := diskUsage(e.opts.target); err == nil {
		e.freeAtStart, e.diskTotal = free, total
	}
	need := uint64(c.size) + e.opts.reserve
	if e.freeAtStart > 0 && need > e.freeAtStart {
		if e.opts.fill {
			e.nSkipped++
			e.report.Event(skipMsg{name: c.name, size: c.size, reason: "won't fit"})
			return false
		}
		e.nSkipped++
		e.stoppedFull = true
		e.report.Event(skipMsg{name: c.name, size: c.size, reason: "won't fit — stopping", fatal: true})
		return true
	}

	// A destination that appeared since vetting (or a resumed run) is skipped,
	// never fatal.
	if !e.opts.force && exists(c.dest) {
		e.nSkipped++
		e.report.Event(skipMsg{name: c.name, size: c.size, reason: "already exists"})
		return false
	}
	if err := os.MkdirAll(filepath.Dir(c.dest), 0o755); err != nil {
		e.nFailed++
		e.report.Event(failMsg{name: c.name, reason: err.Error()})
		return false
	}

	if !e.opts.modest {
		e.report.Event(thumbMsg{name: c.name, art: renderThumb(c.path, thumbCols, thumbRows)})
	}

	if e.sessionStart.IsZero() {
		e.sessionStart = time.Now()
	}
	e.speedo.reset()
	e.lastEmit = time.Time{}
	fileStart := time.Now()
	freeAtStart := e.freeAtStart
	e.index++

	e.report.Event(fileStartMsg{
		name: c.name, path: c.path, dest: c.dest, note: c.note,
		size: c.size, index: e.index,
	})

	onProgress := func(copied int64) {
		now := time.Now()
		if !e.lastEmit.IsZero() && now.Sub(e.lastEmit) < 60*time.Millisecond {
			return
		}
		e.lastEmit = now
		inst := e.speedo.sample(now, copied)
		live := uint64(0)
		if uint64(copied) < freeAtStart {
			live = freeAtStart - uint64(copied)
		}
		e.report.Event(progressMsg{
			copied:    copied,
			total:     c.size,
			instSpeed: inst,
			avgSpeed:  e.sessionAvg(now, copied),
			free:      live,
			diskTotal: e.diskTotal,
		})
	}

	var copyErr error
	for attempt := 0; attempt <= e.opts.retries; attempt++ {
		copyErr = copyOne(e.ctx, c.path, c.dest, c.size, e.opts.verify, onProgress)
		if copyErr == nil {
			break
		}
		if e.ctx.Err() != nil {
			return true
		}
		if attempt < e.opts.retries {
			e.report.Event(failMsg{name: c.name, reason: "retry " + strconv.Itoa(attempt+1) + ": " + copyErr.Error()})
			time.Sleep(250 * time.Millisecond)
		}
	}

	if copyErr != nil {
		if e.ctx.Err() != nil {
			return true
		}
		e.nFailed++
		e.report.Event(failMsg{name: c.name, reason: copyErr.Error()})
		return false
	}

	e.nCopied++
	e.copiedBytes += c.size
	e.sessionBytes += c.size
	e.report.Event(fileDoneMsg{
		name: c.name, dest: c.dest, note: c.note, size: c.size,
		dur: time.Since(fileStart), verified: e.opts.verify,
	})

	if free, total, err := diskUsage(e.opts.target); err == nil {
		e.freeAtStart, e.diskTotal = free, total
		e.report.Event(diskMsg{free: free, diskTotal: total, avgSpeed: e.sessionAvg(time.Now(), 0)})
	}
	return false
}

// verifyCandidate checks one file against its counterpart in the target
// without writing anything. Nothing here can fill the drive, so the only
// reason it stops the run is cancellation.
func (e *engine) verifyCandidate(c *candidate) (stop bool) {
	if !e.opts.modest {
		e.report.Event(thumbMsg{name: c.name, art: renderThumb(c.path, thumbCols, thumbRows)})
	}

	if e.sessionStart.IsZero() {
		e.sessionStart = time.Now()
	}
	e.speedo.reset()
	e.lastEmit = time.Time{}
	fileStart := time.Now()
	e.index++

	e.report.Event(fileStartMsg{
		name: c.name, path: c.path, dest: c.dest, note: c.note,
		size: c.size, index: e.index,
	})

	// Hash mode reads the file twice, once per side. Progress is reported
	// against the file's own size so the bar and the rate describe how fast
	// this file is being cleared, not how many bytes went past the head.
	read := verifyTotal(c.size, e.opts.verify)
	onProgress := func(n int64) {
		now := time.Now()
		if !e.lastEmit.IsZero() && now.Sub(e.lastEmit) < 60*time.Millisecond {
			return
		}
		e.lastEmit = now
		done := n
		if read > 0 {
			done = int64(float64(n) / float64(read) * float64(c.size))
		}
		e.report.Event(progressMsg{
			copied:    done,
			total:     c.size,
			instSpeed: e.speedo.sample(now, done),
			avgSpeed:  e.sessionAvg(now, done),
			free:      e.freeAtStart,
			diskTotal: e.diskTotal,
		})
	}

	err := verifyExisting(e.ctx, c.path, c.dest, c.size, e.opts.verify, onProgress)
	if e.ctx.Err() != nil {
		return true
	}
	if err != nil {
		e.nFailed++
		e.report.Event(failMsg{name: c.name, reason: err.Error()})
		return false
	}

	e.nCopied++
	e.copiedBytes += c.size
	e.sessionBytes += c.size
	e.report.Event(fileDoneMsg{
		name: c.name, dest: c.dest, note: c.note, size: c.size,
		dur: time.Since(fileStart), verified: e.opts.verify,
	})
	return false
}

func exists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
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
