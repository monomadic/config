package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"path/filepath"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

const usageText = `spill — copy files named on stdin to a target until it's full

Usage:
  <paths on stdin> | spill [flags] TARGET_DIR

Reads a list of file paths from stdin (one per line, or NUL-separated with -0)
and copies each into TARGET_DIR, showing a live TUI with two neon progress
bars: the current file (with write speed) and the target drive's remaining
space (with an estimated time to fill). Stops when input ends or the next file
won't fit.

Example:
  fd . /Volumes/src -tf -0 | spill -0 --fill --verify hash /Volumes/backup

Flags:
  -0, --null        Input paths are NUL-separated (pairs with fd/find -print0)
  --fill            When a file won't fit, skip it and keep going until the
                    drive is totally full (default: stop at the first misfit)
  --verify MODE     Verify each copy: "size" or "hash" (default: off)
  --retry N         Extra attempts after a failed copy/verify (default: 2)
  --reserve SIZE    Keep SIZE free on the target, e.g. 1G, 500M (default: 0)
  --force           Overwrite files that already exist in TARGET_DIR
  --modest          Never render thumbnails
  -h, --help        Show this help
`

func main() {
	var (
		nullShort, nullLong bool
		verifyStr           string
		reserveStr          string
		opts                options
	)

	fs := flag.NewFlagSet("spill", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	fs.Usage = func() { fmt.Fprint(os.Stderr, usageText) }
	fs.BoolVar(&nullShort, "0", false, "")
	fs.BoolVar(&nullLong, "null", false, "")
	fs.BoolVar(&opts.fill, "fill", false, "")
	fs.StringVar(&verifyStr, "verify", "", "")
	fs.IntVar(&opts.retries, "retry", 2, "")
	fs.StringVar(&reserveStr, "reserve", "0", "")
	fs.BoolVar(&opts.force, "force", false, "")
	fs.BoolVar(&opts.modest, "modest", false, "")

	if err := fs.Parse(os.Args[1:]); err != nil {
		os.Exit(2)
	}

	opts.null = nullShort || nullLong
	if opts.retries < 0 {
		opts.retries = 0
	}

	switch verifyStr {
	case "", "off", "none":
		opts.verify = verifyNone
	case "size":
		opts.verify = verifySize
	case "hash":
		opts.verify = verifyHash
	default:
		fatalf("invalid --verify %q (want size or hash)", verifyStr)
	}

	reserve, err := parseSize(reserveStr)
	if err != nil {
		fatalf("%v", err)
	}
	opts.reserve = reserve

	opts.target = fs.Arg(0)
	if opts.target == "" {
		fs.Usage()
		os.Exit(2)
	}
	if abs, err := filepath.Abs(opts.target); err == nil {
		opts.target = abs
	}
	if info, err := os.Stat(opts.target); err != nil || !info.IsDir() {
		fatalf("target is not a directory: %s", opts.target)
	}

	if isTTY(os.Stdin) {
		fmt.Fprintln(os.Stderr, "spill: expects a list of file paths on stdin")
		fs.Usage()
		os.Exit(2)
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	if isTTY(os.Stdout) {
		os.Exit(runTUI(ctx, cancel, opts))
	}
	os.Exit(runPlain(ctx, cancel, opts))
}

func runTUI(ctx context.Context, cancel context.CancelFunc, opts options) int {
	p := tea.NewProgram(newModel(opts, cancel), tea.WithAltScreen())

	go func() {
		eng := newEngine(ctx, opts, teaReporter{p: p})
		eng.run(os.Stdin)
	}()

	fm, err := p.Run()
	cancel()
	if err != nil {
		fmt.Fprintf(os.Stderr, "spill: %v\n", err)
		return 1
	}
	if m, ok := fm.(model); ok && (m.sum.failed > 0 || m.nFailed > 0) {
		return 1
	}
	return 0
}

func runPlain(ctx context.Context, cancel context.CancelFunc, opts options) int {
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, os.Interrupt)
	go func() {
		<-sig
		cancel()
	}()

	pr := &plainReporter{target: opts.target}
	eng := newEngine(ctx, opts, pr)
	sum := eng.run(os.Stdin)
	if sum.failed > 0 {
		return 1
	}
	return 0
}

// plainReporter renders progress as terse lines when stdout is not a terminal.
// Successful destination paths go to stdout (so `spill … | xargs` works);
// status goes to stderr.
type plainReporter struct {
	target   string
	curName  string
	lastEmit time.Time
}

func (r *plainReporter) Event(msg any) {
	switch m := msg.(type) {
	case fileStartMsg:
		r.curName = m.name
		fmt.Fprintf(os.Stderr, "→ %s (%s)\n", m.name, humanBytes(m.size))
	case progressMsg:
		now := time.Now()
		if now.Sub(r.lastEmit) < 500*time.Millisecond {
			return
		}
		r.lastEmit = now
		pct := 0
		if m.total > 0 {
			pct = int(float64(m.copied) / float64(m.total) * 100)
		}
		fmt.Fprintf(os.Stderr, "\r  %s %3d%%  %s  free %s\033[K",
			truncate(r.curName, 32), pct, humanRate(m.instSpeed), humanUBytes(m.free))
	case fileDoneMsg:
		fmt.Fprintf(os.Stderr, "\r\033[K✓ %s (%s)\n", m.name, humanBytes(m.size))
		fmt.Fprintln(os.Stdout, filepath.Join(r.target, m.name))
	case skipMsg:
		fmt.Fprintf(os.Stderr, "\r\033[K⤳ %s — %s\n", m.name, m.reason)
	case failMsg:
		fmt.Fprintf(os.Stderr, "\r\033[K✕ %s — %s\n", m.name, m.reason)
	case doneMsg:
		s := m.summary
		reason := "input exhausted"
		if s.stoppedFull {
			reason = "drive full"
		}
		fmt.Fprintf(os.Stderr, "\ndone: %d copied, %d skipped, %d failed · %s in %s · %s\n",
			s.copied, s.skipped, s.failed, humanBytes(s.copiedBytes), humanDuration(s.elapsed), reason)
	}
}

func isTTY(f *os.File) bool {
	fi, err := f.Stat()
	if err != nil {
		return false
	}
	return fi.Mode()&os.ModeCharDevice != 0
}

func fatalf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, "spill: "+format+"\n", args...)
	os.Exit(2)
}
