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
	target     string       // destination directory
	strategy   strategyKind // which files to copy, and in what order
	null       bool         // input paths are NUL-separated instead of newline
	fill       bool         // skip files that don't fit and keep going until full
	flatten    bool         // copy every file into the target root, ignoring input structure
	verify     verifyMode   // post-copy verification
	verifyOnly bool         // check existing target files instead of copying
	retries    int          // extra attempts after the first on copy/verify failure
	reserve    uint64       // bytes of headroom to keep free on the target
	force      bool         // overwrite existing destination files
	modest     bool         // never render thumbnails
}

// verbing and verbed name what the run does to a file, so the same progress
// output reads correctly for a copy run and a --verify-only run.
func (o options) verbing() string {
	if o.verifyOnly {
		return "verifying"
	}
	return "copying"
}

func (o options) verbed() string {
	if o.verifyOnly {
		return "verified"
	}
	return "copied"
}

// The engine reports progress to a Reporter as a stream of these events. Both
// the Bubble Tea UI and the plain-text fallback consume the same event types.
type Reporter interface {
	Event(any)
}

type fileStartMsg struct {
	name  string
	path  string
	dest  string
	note  string // why the strategy picked this file, if it said
	size  int64
	index int
}

// scanMsg reports the inspection pass a strategy runs before (or, for the
// streaming strategies, between) copies. total is 0 while streaming, where
// the size of the input isn't known yet.
type scanMsg struct {
	name  string
	done  int
	total int
}

// filterMsg marks a file the strategy declined to copy. These are counted
// apart from skips: nothing was wrong with the file, it just didn't match.
type filterMsg struct {
	name   string
	reason string
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
	dest     string
	note     string
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
	filtered    int
	failed      int
	elapsed     time.Duration
	free        uint64
	diskTotal   uint64
	stoppedFull bool // stopped because the next file didn't fit (no --fill)
}
