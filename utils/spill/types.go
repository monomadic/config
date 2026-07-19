package main

import "time"

// verifyMode selects how a completed copy is checked before it counts as done.
type verifyMode int

const (
	verifyNone verifyMode = iota
	verifySize
	verifyHash
)

func (v verifyMode) String() string {
	switch v {
	case verifySize:
		return "size"
	case verifyHash:
		return "hash"
	default:
		return "off"
	}
}

// options holds the parsed command-line configuration for a run.
type options struct {
	target  string     // destination directory
	null    bool       // input paths are NUL-separated instead of newline
	fill    bool       // skip files that don't fit and keep going until full
	verify  verifyMode // post-copy verification
	retries int        // extra attempts after the first on copy/verify failure
	reserve uint64     // bytes of headroom to keep free on the target
	force   bool       // overwrite existing destination files
	modest  bool       // never render thumbnails
}

// The engine reports progress to a Reporter as a stream of these events. Both
// the Bubble Tea UI and the plain-text fallback consume the same event types.
type Reporter interface {
	Event(any)
}

type fileStartMsg struct {
	name  string
	path  string
	size  int64
	index int
}

type thumbMsg struct {
	name string // the file the art belongs to, so stale art is ignored
	art  string // pre-rendered (chafa) thumbnail, or "" if none
}

// progressMsg is emitted repeatedly during a copy. Disk figures are the live
// estimate (free-at-file-start minus bytes written) so the disk bar animates
// smoothly; an authoritative statfs refresh lands at each file boundary.
type progressMsg struct {
	copied    int64
	total     int64
	instSpeed float64 // bytes/sec, smoothed
	avgSpeed  float64 // bytes/sec over the whole session
	free      uint64
	diskTotal uint64
}

type fileDoneMsg struct {
	name     string
	size     int64
	dur      time.Duration
	verified verifyMode
}

type skipMsg struct {
	name   string
	size   int64
	reason string
	fatal  bool // true when a non-fitting file ended the run (no --fill)
}

type failMsg struct {
	name   string
	reason string
}

// diskMsg carries an authoritative statfs reading between files.
type diskMsg struct {
	free      uint64
	diskTotal uint64
	avgSpeed  float64
}

type doneMsg struct {
	summary summary
}

type summary struct {
	copied      int
	copiedBytes int64
	skipped     int
	failed      int
	elapsed     time.Duration
	free        uint64
	diskTotal   uint64
	stoppedFull bool // stopped because the next file didn't fit (no --fill)
}
