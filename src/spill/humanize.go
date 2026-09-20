package main

import (
	"fmt"
	"math"
	"strconv"
	"strings"
	"time"
)

// humanBytes formats a byte count with binary units (KiB steps but SI-ish
// labels), matching the terse style of the media-audit TUI.
func humanBytes(b int64) string {
	return humanUBytes(uint64(max64(b, 0)))
}

func humanUBytes(b uint64) string {
	const unit = 1024.0
	units := []string{"B", "KB", "MB", "GB", "TB", "PB"}
	f := float64(b)
	i := 0
	for f >= unit && i < len(units)-1 {
		f /= unit
		i++
	}
	switch {
	case i == 0:
		return fmt.Sprintf("%d %s", b, units[i])
	case f >= 100:
		return fmt.Sprintf("%.0f %s", f, units[i])
	case f >= 10:
		return fmt.Sprintf("%.1f %s", f, units[i])
	default:
		return fmt.Sprintf("%.2f %s", f, units[i])
	}
}

// humanRate renders a bytes/second figure as e.g. "142 MB/s".
func humanRate(bytesPerSec float64) string {
	if bytesPerSec <= 0 || math.IsNaN(bytesPerSec) || math.IsInf(bytesPerSec, 0) {
		return "—"
	}
	return humanUBytes(uint64(bytesPerSec)) + "/s"
}

// humanDuration renders a short clock like "4m12s" or "1h03m".
func humanDuration(d time.Duration) string {
	if d < 0 {
		d = 0
	}
	s := int(d.Seconds() + 0.5)
	h := s / 3600
	s -= h * 3600
	m := s / 60
	s -= m * 60
	switch {
	case h > 0:
		return fmt.Sprintf("%dh%02dm", h, m)
	case m > 0:
		return fmt.Sprintf("%dm%02ds", m, s)
	default:
		return fmt.Sprintf("%ds", s)
	}
}

// humanETA renders an estimated time from a remaining byte count and a rate,
// returning "—" when the rate is unusable.
func humanETA(remaining float64, bytesPerSec float64) string {
	if bytesPerSec <= 0 || remaining <= 0 || math.IsNaN(bytesPerSec) || math.IsInf(bytesPerSec, 0) {
		return "—"
	}
	return humanDuration(time.Duration(remaining/bytesPerSec) * time.Second)
}

// parseSize accepts a raw byte count or a suffixed value like "1G", "500M",
// "2.5g", "1GiB". Binary multipliers throughout.
func parseSize(s string) (uint64, error) {
	s = strings.TrimSpace(s)
	if s == "" {
		return 0, nil
	}
	mult := 1.0
	lower := strings.ToLower(s)
	lower = strings.TrimSuffix(lower, "b")
	lower = strings.TrimSuffix(lower, "i")
	switch {
	case strings.HasSuffix(lower, "k"):
		mult, lower = 1<<10, strings.TrimSuffix(lower, "k")
	case strings.HasSuffix(lower, "m"):
		mult, lower = 1<<20, strings.TrimSuffix(lower, "m")
	case strings.HasSuffix(lower, "g"):
		mult, lower = 1<<30, strings.TrimSuffix(lower, "g")
	case strings.HasSuffix(lower, "t"):
		mult, lower = 1<<40, strings.TrimSuffix(lower, "t")
	}
	lower = strings.TrimSpace(lower)
	v, err := strconv.ParseFloat(lower, 64)
	if err != nil {
		return 0, fmt.Errorf("invalid size %q", s)
	}
	if v < 0 {
		return 0, fmt.Errorf("negative size %q", s)
	}
	return uint64(v * mult), nil
}

func max64(a, b int64) int64 {
	if a > b {
		return a
	}
	return b
}
