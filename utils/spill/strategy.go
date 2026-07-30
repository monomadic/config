package main

import (
	"context"
	"fmt"
	"sort"
	"strings"
	"time"
)

// A strategy decides which of the paths on stdin are worth copying, and in
// what order. Two shapes exist:
//
//   - sorting strategies drain stdin first, inspect every candidate, then copy
//     the survivors best-first;
//   - streaming strategies judge each path as it arrives and copy immediately,
//     so they work on a list that is still being produced.
type strategyKind int

// stratNone is the zero value, and so the default: no inspection, no sort,
// nothing to wait for.
const (
	stratNone strategyKind = iota
	stratHighQuality
	stratHighestQuality
	stratGoodQuality
	stratLatest
	stratAudit
	stratURLMissing
)

type strategySpec struct {
	name  string
	sorts bool // needs the whole input before the first copy
	desc  string
}

// strategyOrder is the display order for --help; the default comes first.
var strategyOrder = []strategyKind{
	stratNone, stratHighQuality, stratHighestQuality, stratGoodQuality,
	stratLatest, stratAudit, stratURLMissing,
}

var strategySpecs = map[strategyKind]strategySpec{
	stratNone:           {"none", false, "every file, in the order given (default)"},
	stratHighQuality:    {"high-quality", true, "3★+, 1080p+, 60fps+ — best first"},
	stratHighestQuality: {"highest-quality", true, "4★+, 4K+, 60fps+ — best first"},
	stratGoodQuality:    {"good-quality", true, "3★+, 1080p+, 30fps+ — best first"},
	stratLatest:         {"latest", true, "newest files first, no filtering"},
	stratAudit:          {"audit", false, "files with a black intro or no faststart"},
	stratURLMissing:     {"url-missing", false, "files with no embedded source URL"},
}

// qualityBar is the threshold a file must clear under a quality strategy.
// Frame rates are compared with a 1fps slack so 59.94 and 29.97 pass.
type qualityBar struct {
	rating int
	side   int
	fps    float64
}

var qualityBars = map[strategyKind]qualityBar{
	stratHighQuality:    {3, 1080, 60},
	stratHighestQuality: {4, 2160, 60},
	stratGoodQuality:    {3, 1080, 30},
}

func (k strategyKind) spec() strategySpec { return strategySpecs[k] }

func (k strategyKind) String() string { return strategySpecs[k].name }

func parseStrategy(s string) (strategyKind, error) {
	for k, spec := range strategySpecs {
		if spec.name == s {
			return k, nil
		}
	}
	names := make([]string, 0, len(strategyOrder))
	for _, k := range strategyOrder {
		names = append(names, strategySpecs[k].name)
	}
	return 0, fmt.Errorf("invalid --strategy %q (want %s)", s, strings.Join(names, ", "))
}

// inspects reports whether the strategy looks inside the files at all. The
// ones that don't (none, latest) get everything they need from the stat that
// vetting already did, so there is no inspection pass to run or report.
func (k strategyKind) inspects() bool { return k.requiredTool() != "" }

// requiredTool is the external command a strategy cannot work without.
func (k strategyKind) requiredTool() string {
	switch k {
	case stratHighQuality, stratHighestQuality, stratGoodQuality, stratURLMissing:
		return "ffprobe"
	case stratAudit:
		return "ffmpeg"
	}
	return ""
}

// candidate is one input path plus whatever the active strategy needed to
// learn about it in order to judge and rank it.
type candidate struct {
	path string // as given on stdin
	name string // display name: the relative path, or the basename when flattened
	dest string // absolute destination path
	size int64
	mod  time.Time

	// filled by prepare(), per strategy
	rating   int
	side     int
	fps      float64
	hasURL   bool
	probeErr error
	note     string // why this file was picked, shown in the log
}

// prepare runs whatever inspection the strategy needs before it can judge a
// candidate. It is the expensive half (ffprobe/ffmpeg) and is safe to run
// concurrently on distinct candidates.
func prepare(ctx context.Context, k strategyKind, c *candidate) {
	switch k {
	case stratHighQuality, stratHighestQuality, stratGoodQuality:
		c.rating = ratingFromName(c.path)
		if !isVideoPath(c.path) {
			c.probeErr = errNotVideo
			return
		}
		c.side, c.fps, c.probeErr = probeVideo(ctx, c.path)

	case stratURLMissing:
		url, err := probeSourceURL(ctx, c.path)
		if err != nil {
			c.probeErr = err
			return
		}
		c.hasURL = url != ""
		if !c.hasURL {
			c.note = "no source URL"
		}

	case stratAudit:
		if isFaststartPath(c.path) {
			switch faststartState(c.path) {
			case faststartMissing:
				c.note = "no faststart"
				return
			case faststartUnknown:
				c.probeErr = errFaststartUnknown
				return
			}
		}
		if !isVideoPath(c.path) {
			c.probeErr = errNotVideo
			return
		}
		secs, err := blackIntroSeconds(ctx, c.path)
		if err != nil {
			c.probeErr = err
			return
		}
		if secs > 0 {
			c.note = fmt.Sprintf("%.1fs black intro", secs)
		}
	}
}

var (
	errNotVideo         = fmt.Errorf("not a video")
	errFaststartUnknown = fmt.Errorf("faststart inconclusive")
)

// admit reports whether a prepared candidate should be copied, and when it
// should not, the reason to show for leaving it out.
func admit(k strategyKind, c *candidate) (bool, string) {
	switch k {
	case stratHighQuality, stratHighestQuality, stratGoodQuality:
		bar := qualityBars[k]
		if c.rating < bar.rating {
			if c.rating == 0 {
				return false, fmt.Sprintf("unrated (need %d★)", bar.rating)
			}
			return false, fmt.Sprintf("%d★ (need %d★)", c.rating, bar.rating)
		}
		if c.probeErr != nil {
			return false, c.probeErr.Error()
		}
		if c.side < bar.side {
			return false, fmt.Sprintf("%dp (need %dp)", c.side, bar.side)
		}
		if c.fps < bar.fps-1 {
			return false, fmt.Sprintf("%.0ffps (need %.0ffps)", c.fps, bar.fps)
		}
		c.note = fmt.Sprintf("%d★ %dp %.0ffps", c.rating, c.side, c.fps)
		return true, ""

	case stratURLMissing:
		if c.probeErr != nil {
			return false, c.probeErr.Error()
		}
		if c.hasURL {
			return false, "has a source URL"
		}
		return true, ""

	case stratAudit:
		if c.note != "" {
			return true, ""
		}
		if c.probeErr != nil {
			return false, c.probeErr.Error()
		}
		return false, "nothing to fix"
	}
	return true, ""
}

// sortCandidates orders the admitted files for a sorting strategy.
func sortCandidates(k strategyKind, cands []*candidate) {
	switch k {
	case stratHighQuality, stratHighestQuality, stratGoodQuality:
		sort.SliceStable(cands, func(i, j int) bool {
			a, b := cands[i], cands[j]
			if a.rating != b.rating {
				return a.rating > b.rating
			}
			if a.side != b.side {
				return a.side > b.side
			}
			if a.fps != b.fps {
				return a.fps > b.fps
			}
			if a.size != b.size {
				return a.size > b.size
			}
			return a.name < b.name
		})

	case stratLatest:
		sort.SliceStable(cands, func(i, j int) bool {
			a, b := cands[i], cands[j]
			if !a.mod.Equal(b.mod) {
				return a.mod.After(b.mod)
			}
			return a.name < b.name
		})
	}
}
