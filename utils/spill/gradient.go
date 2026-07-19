package main

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/lipgloss"
)

type rgb struct{ r, g, b float64 }

func (c rgb) hex() string {
	return fmt.Sprintf("#%02X%02X%02X", clamp8(c.r), clamp8(c.g), clamp8(c.b))
}

func clamp8(v float64) uint8 {
	if v < 0 {
		v = 0
	}
	if v > 255 {
		v = 255
	}
	return uint8(v + 0.5)
}

// Neon stops. The active-copy bar runs hot-pink → violet → electric-cyan; the
// disk-fill bar runs teal → blue → hot-pink so a nearly-full drive reads warm.
var (
	copyStops = []rgb{{0xFF, 0x2E, 0xC0}, {0x8A, 0x5C, 0xFF}, {0x1E, 0xE6, 0xFF}}
	diskStops = []rgb{{0x22, 0xF5, 0xC8}, {0x4F, 0x9C, 0xFF}, {0xFF, 0x3C, 0x8A}}
	emptyCell = lipgloss.NewStyle().Foreground(lipgloss.Color("#2A2E3A"))
)

func lerp(a, b rgb, t float64) rgb {
	return rgb{
		r: a.r + (b.r-a.r)*t,
		g: a.g + (b.g-a.g)*t,
		b: a.b + (b.b-a.b)*t,
	}
}

// gradientAt samples a multi-stop gradient at t in [0,1].
func gradientAt(stops []rgb, t float64) rgb {
	if len(stops) == 1 {
		return stops[0]
	}
	if t <= 0 {
		return stops[0]
	}
	if t >= 1 {
		return stops[len(stops)-1]
	}
	scaled := t * float64(len(stops)-1)
	i := int(scaled)
	if i >= len(stops)-1 {
		return stops[len(stops)-1]
	}
	return lerp(stops[i], stops[i+1], scaled-float64(i))
}

// gradientBar renders a filled/empty bar of the given cell width, colouring the
// filled run along the gradient so the neon sweep is visible edge to edge.
func gradientBar(width int, ratio float64, stops []rgb) string {
	if width < 1 {
		width = 1
	}
	if ratio < 0 {
		ratio = 0
	}
	if ratio > 1 {
		ratio = 1
	}
	filled := int(ratio*float64(width) + 0.5)
	if filled > width {
		filled = width
	}
	var b strings.Builder
	for i := 0; i < filled; i++ {
		t := 0.0
		if width > 1 {
			t = float64(i) / float64(width-1)
		}
		style := lipgloss.NewStyle().Foreground(lipgloss.Color(gradientAt(stops, t).hex()))
		b.WriteString(style.Render("█"))
	}
	if width-filled > 0 {
		b.WriteString(emptyCell.Render(strings.Repeat("░", width-filled)))
	}
	return b.String()
}

// gradientText colours each rune of s along the gradient — used for the title.
func gradientText(s string, stops []rgb) string {
	runes := []rune(s)
	var b strings.Builder
	for i, r := range runes {
		t := 0.0
		if len(runes) > 1 {
			t = float64(i) / float64(len(runes)-1)
		}
		style := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color(gradientAt(stops, t).hex()))
		b.WriteString(style.Render(string(r)))
	}
	return b.String()
}
