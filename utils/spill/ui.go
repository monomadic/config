package main

import (
	"context"
	"fmt"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

var (
	stDim   = lipgloss.NewStyle().Foreground(lipgloss.Color("#6B7280"))
	stLabel = lipgloss.NewStyle().Foreground(lipgloss.Color("#9AA4B2"))
	stName  = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#E8ECF4"))
	stOK    = lipgloss.NewStyle().Foreground(lipgloss.Color("#3BE38B"))
	stWarn  = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFC24B"))
	stErr   = lipgloss.NewStyle().Foreground(lipgloss.Color("#FF5C7A"))
	stPct   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#1EE6FF"))
	stFree  = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#FF6FB5"))
	stSpeed = lipgloss.NewStyle().Foreground(lipgloss.Color("#8A5CFF"))
)

const maxLog = 6

type model struct {
	opts   options
	cancel context.CancelFunc

	w, h int

	curName string
	curSize int64
	copied  int64
	total   int64
	inst    float64
	avg     float64

	art     string
	artName string

	free      uint64
	diskTotal uint64

	nCopied     int
	copiedBytes int64
	nSkipped    int
	nFailed     int

	logLines []string

	done bool
	sum  summary
}

func newModel(opts options, cancel context.CancelFunc) model {
	return model{opts: opts, cancel: cancel}
}

func (m model) Init() tea.Cmd { return nil }

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.w, m.h = msg.Width, msg.Height
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q", "esc":
			if m.cancel != nil {
				m.cancel()
			}
			return m, tea.Quit
		}
		if m.done {
			return m, tea.Quit
		}
		return m, nil

	case fileStartMsg:
		m.curName, m.curSize = msg.name, msg.size
		m.copied, m.total = 0, msg.size
		m.inst = 0
		if m.artName != msg.name {
			m.art = ""
		}
		return m, nil

	case thumbMsg:
		m.art, m.artName = msg.art, msg.name
		return m, nil

	case progressMsg:
		m.copied, m.total = msg.copied, msg.total
		m.inst, m.avg = msg.instSpeed, msg.avgSpeed
		if msg.diskTotal > 0 {
			m.free, m.diskTotal = msg.free, msg.diskTotal
		}
		return m, nil

	case fileDoneMsg:
		m.nCopied++
		m.copiedBytes += msg.size
		m.copied = m.total
		tag := ""
		if msg.verified != verifyNone {
			tag = stDim.Render(" ✓" + msg.verified.String())
		}
		m.pushLog(stOK.Render("✓ ") + truncate(msg.name, 40) + " " +
			stDim.Render("("+humanBytes(msg.size)+", "+humanRate(float64(msg.size)/msg.dur.Seconds())+")") + tag)
		return m, nil

	case skipMsg:
		style := stWarn
		if msg.fatal {
			style = stErr
		}
		m.pushLog(style.Render("⤳ ") + truncate(msg.name, 40) + " " + stDim.Render(msg.reason))
		return m, nil

	case failMsg:
		m.pushLog(stErr.Render("✕ ") + truncate(msg.name, 40) + " " + stDim.Render(truncate(msg.reason, 48)))
		return m, nil

	case diskMsg:
		if msg.diskTotal > 0 {
			m.free, m.diskTotal = msg.free, msg.diskTotal
		}
		if msg.avgSpeed > 0 {
			m.avg = msg.avgSpeed
		}
		return m, nil

	case doneMsg:
		m.done, m.sum = true, msg.summary
		return m, nil
	}
	return m, nil
}

func (m *model) pushLog(line string) {
	m.logLines = append(m.logLines, line)
	if len(m.logLines) > maxLog {
		m.logLines = m.logLines[len(m.logLines)-maxLog:]
	}
}

func (m model) width() int {
	if m.w > 0 {
		return m.w
	}
	return 90
}

func (m model) barWidth() int {
	w := m.width() - 46
	if w < 12 {
		w = 12
	}
	if w > 64 {
		w = 64
	}
	return w
}

func (m model) View() string {
	var b strings.Builder

	title := gradientText("SPILL", copyStops)
	b.WriteString(title + "  " + stDim.Render("→ "+m.opts.target) + "\n\n")

	// Current-file readout, optionally beside a thumbnail.
	info := m.currentBlock()
	if m.art != "" && !m.opts.modest {
		b.WriteString(lipgloss.JoinHorizontal(lipgloss.Top, m.art, "  ", info) + "\n")
	} else {
		b.WriteString(info + "\n")
	}

	b.WriteString(m.diskBlock() + "\n")
	b.WriteString(m.talliesLine() + "\n\n")

	if len(m.logLines) > 0 {
		b.WriteString(strings.Join(m.logLines, "\n") + "\n")
	}

	b.WriteString("\n")
	if m.done {
		b.WriteString(m.summaryLine() + "\n")
		b.WriteString(stDim.Render("done · press q to exit"))
	} else {
		b.WriteString(stDim.Render("copying · q to stop"))
	}
	return b.String()
}

func (m model) currentBlock() string {
	var b strings.Builder

	name := m.curName
	if name == "" {
		name = "…"
	}
	b.WriteString(stName.Render(truncate(name, m.width()-thumbCols-8)) + "\n")

	ratio := 0.0
	if m.total > 0 {
		ratio = float64(m.copied) / float64(m.total)
	}
	pct := fmt.Sprintf("%3d%%", int(ratio*100))
	bar := gradientBar(m.barWidth(), ratio, copyStops)
	sizeInfo := fmt.Sprintf("%s / %s", humanBytes(m.copied), humanBytes(m.curSize))
	remain := float64(m.total - m.copied)

	b.WriteString(stPct.Render(pct) + " " + bar + "\n")
	b.WriteString(stLabel.Render("  file ") + stSpeed.Render(humanRate(m.inst)) +
		stDim.Render("  "+sizeInfo) +
		stDim.Render("  eta "+humanETA(remain, m.inst)))
	return b.String()
}

func (m model) diskBlock() string {
	var b strings.Builder

	usedRatio := 0.0
	if m.diskTotal > 0 {
		usedRatio = 1 - float64(m.free)/float64(m.diskTotal)
	}
	pct := fmt.Sprintf("%3d%%", int(usedRatio*100))
	bar := gradientBar(m.barWidth(), usedRatio, diskStops)

	b.WriteString("\n")
	b.WriteString(stLabel.Render("DISK") + "\n")
	b.WriteString(stPct.Render(pct) + " " + bar + "\n")
	b.WriteString(stLabel.Render("  free ") + stFree.Render(humanUBytes(m.free)) +
		stDim.Render("  avg "+humanRate(m.avg)) +
		stDim.Render("  ~"+humanETA(float64(m.free), m.avg)+" to full"))
	return b.String()
}

func (m model) talliesLine() string {
	return "\n" + stOK.Render(fmt.Sprintf("✓ %d copied", m.nCopied)) + stDim.Render(" · ") +
		stWarn.Render(fmt.Sprintf("⤳ %d skipped", m.nSkipped)) + stDim.Render(" · ") +
		stErr.Render(fmt.Sprintf("✕ %d failed", m.nFailed)) + stDim.Render(" · ") +
		stDim.Render(humanBytes(m.copiedBytes)+" written")
}

func (m model) summaryLine() string {
	reason := "input exhausted"
	if m.sum.stoppedFull {
		reason = "drive full (next file didn't fit)"
	}
	return stName.Render("Done: ") +
		stOK.Render(fmt.Sprintf("%d copied", m.sum.copied)) + stDim.Render(" · ") +
		stWarn.Render(fmt.Sprintf("%d skipped", m.sum.skipped)) + stDim.Render(" · ") +
		stErr.Render(fmt.Sprintf("%d failed", m.sum.failed)) + stDim.Render(" · ") +
		stDim.Render(humanBytes(m.sum.copiedBytes)+" in "+humanDuration(m.sum.elapsed)+" · "+reason)
}

// teaReporter forwards engine events into the Bubble Tea program.
type teaReporter struct{ p *tea.Program }

func (r teaReporter) Event(msg any) { r.p.Send(msg) }

func truncate(s string, n int) string {
	if n < 1 {
		n = 1
	}
	r := []rune(s)
	if len(r) <= n {
		return s
	}
	if n == 1 {
		return "…"
	}
	return string(r[:n-1]) + "…"
}
