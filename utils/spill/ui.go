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

	curNote string

	art     string
	artName string

	scanName  string
	scanDone  int
	scanTotal int
	scanning  bool

	free      uint64
	diskTotal uint64

	nCopied     int
	copiedBytes int64
	nSkipped    int
	nFiltered   int
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

	case scanMsg:
		m.scanning = true
		m.scanName, m.scanDone, m.scanTotal = msg.name, msg.done, msg.total
		return m, nil

	case filterMsg:
		m.nFiltered++
		return m, nil

	case fileStartMsg:
		m.scanning = false
		m.curName, m.curSize, m.curNote = msg.name, msg.size, msg.note
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
		if msg.note != "" {
			tag += stDim.Render(" · " + msg.note)
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
	b.WriteString(title + "  " + stLabel.Render(m.opts.strategy.String()) +
		stDim.Render("  → "+m.opts.target) + "\n\n")

	// Current-file readout, optionally beside a thumbnail.
	info := m.currentBlock()
	if m.art != "" && !m.opts.modest {
		b.WriteString(lipgloss.JoinHorizontal(lipgloss.Top, m.art, "  ", info) + "\n")
	} else {
		b.WriteString(info + "\n")
	}

	if line := m.scanLine(); line != "" {
		b.WriteString(line + "\n")
	}

	// Nothing is being written, so the fill-the-drive bar has nothing to say.
	if !m.opts.verifyOnly {
		b.WriteString(m.diskBlock() + "\n")
	}
	b.WriteString(m.talliesLine() + "\n\n")

	if len(m.logLines) > 0 {
		b.WriteString(strings.Join(m.logLines, "\n") + "\n")
	}

	b.WriteString("\n")
	if m.done {
		b.WriteString(m.summaryLine() + "\n")
		b.WriteString(stDim.Render("done · press q to exit"))
	} else {
		b.WriteString(stDim.Render(m.opts.verbing() + " · q to stop"))
	}
	return b.String()
}

// scanLine shows the strategy's inspection pass: a progress count while a
// sorting strategy is sizing up the whole list, or the file being examined
// between copies under a streaming strategy.
func (m model) scanLine() string {
	if m.done || !m.scanning {
		return ""
	}
	label := "inspecting"
	if m.scanTotal > 0 {
		label = fmt.Sprintf("inspecting %d/%d", m.scanDone, m.scanTotal)
	}
	return "\n" + stSpeed.Render("⋯ "+label) + " " +
		stDim.Render(truncate(m.scanName, m.width()-thumbCols-24))
}

func (m model) currentBlock() string {
	var b strings.Builder

	name := m.curName
	if name == "" {
		name = "…"
	}
	head := stName.Render(truncate(name, m.width()-thumbCols-8))
	if m.curNote != "" {
		head += stDim.Render("  " + m.curNote)
	}
	b.WriteString(head + "\n")

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
	return "\n" + stOK.Render(fmt.Sprintf("✓ %d %s", m.nCopied, m.opts.verbed())) + stDim.Render(" · ") +
		stWarn.Render(fmt.Sprintf("⤳ %d skipped", m.nSkipped)) + stDim.Render(" · ") +
		stDim.Render(fmt.Sprintf("⊘ %d filtered", m.nFiltered)) + stDim.Render(" · ") +
		stErr.Render(fmt.Sprintf("✕ %d failed", m.nFailed)) + stDim.Render(" · ") +
		stDim.Render(humanBytes(m.copiedBytes)+" "+m.bytesLabel())
}

func (m model) bytesLabel() string {
	if m.opts.verifyOnly {
		return "checked"
	}
	return "written"
}

func (m model) summaryLine() string {
	reason := "input exhausted"
	if m.sum.stoppedFull {
		reason = "drive full (next file didn't fit)"
	}
	if m.sum.cancelled {
		reason = "cancelled — files remain uncopied"
	}
	head := stName.Render("Done: ")
	if m.sum.cancelled {
		head = stErr.Render("Stopped: ")
	}
	return head +
		stOK.Render(fmt.Sprintf("%d %s", m.sum.copied, m.opts.verbed())) + stDim.Render(" · ") +
		stWarn.Render(fmt.Sprintf("%d skipped", m.sum.skipped)) + stDim.Render(" · ") +
		stDim.Render(fmt.Sprintf("%d filtered", m.sum.filtered)) + stDim.Render(" · ") +
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
