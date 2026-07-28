package main

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// field indices — the four text inputs, then the rating row
const (
	fTitle = iota
	fActors
	fChannel
	fOrigin
	fTags
	fRating
	fCount
)

var fieldLabels = [fCount]string{"title", "actors", "channel", "origin", "tags", "rating"}

var (
	stNeon   = lipgloss.NewStyle().Foreground(lipgloss.Color("46")) // neon green bar
	stDim    = lipgloss.NewStyle().Faint(true)
	stBold   = lipgloss.NewStyle().Bold(true)
	stAccent = lipgloss.NewStyle().Foreground(lipgloss.Color("39"))
	stFile   = lipgloss.NewStyle().Foreground(lipgloss.Color("51")).Bold(true)
	stOK     = lipgloss.NewStyle().Foreground(lipgloss.Color("2"))
	stWarn   = lipgloss.NewStyle().Foreground(lipgloss.Color("3"))
	stErr    = lipgloss.NewStyle().Foreground(lipgloss.Color("1"))
	stStar   = lipgloss.NewStyle().Foreground(lipgloss.Color("3"))
	stLabel  = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))
	stBorder = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))
)

type model struct {
	pf      *PreflightInfo
	dl      *Downloader
	url     string
	initial Meta // snapshot for the "touched" test

	inputs [fTags + 1]textinput.Model
	focus  int
	rating int

	pct                         int
	dlStr, totStr, spdStr, eta  string
	status, phase               string
	paused                      bool
	thumb                       string

	done      bool
	exitCode  int
	finalReal string
	cancelled bool

	width, height int
}

func newModel(pf *PreflightInfo, dl *Downloader, url string) model {
	m := model{pf: pf, dl: dl, url: url, initial: pf.Meta,
		status: "starting", dlStr: "—", totStr: "—", spdStr: "—", eta: "—",
		width: 80, height: 24}
	seed := [fTags + 1]string{
		pf.Meta.Title,
		strings.Join(pf.Meta.Actors, ", "),
		pf.Meta.Channel,
		pf.Meta.Origin,
		strings.Join(pf.Meta.Tags, " "),
	}
	for i := range m.inputs {
		ti := textinput.New()
		ti.Prompt = ""
		ti.SetValue(seed[i])
		ti.Blur()
		m.inputs[i] = ti
	}
	m.inputs[fTitle].Focus()
	return m
}

func (m model) Init() tea.Cmd { return textinput.Blink }

// meta reads the current form state back into a Meta.
func (m *model) meta() Meta {
	return Meta{
		Title:   strings.TrimSpace(m.inputs[fTitle].Value()),
		Actors:  SplitList(m.inputs[fActors].Value()),
		Channel: strings.TrimSpace(m.inputs[fChannel].Value()),
		Origin:  strings.TrimSpace(m.inputs[fOrigin].Value()),
		Tags:    SplitTags(m.inputs[fTags].Value()),
		Rating:  m.rating,
	}
}

func (m *model) setFocus(f int) {
	m.focus = ((f % fCount) + fCount) % fCount
	for i := range m.inputs {
		if i == m.focus {
			m.inputs[i].Focus()
		} else {
			m.inputs[i].Blur()
		}
	}
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width, m.height = msg.Width, msg.Height
		return m, nil

	case progressMsg:
		m.pct, m.dlStr, m.totStr, m.spdStr, m.eta, m.status =
			msg.Pct, msg.DL, msg.Tot, msg.Spd, msg.Eta, msg.Status
		return m, nil
	case finalPathMsg:
		m.finalReal = string(msg)
		return m, nil
	case logMsg:
		if s := string(msg); len(s) > 0 {
			if len(s) > 72 {
				s = s[:72]
			}
			m.phase = s
		}
		return m, nil
	case thumbMsg:
		m.thumb = string(msg)
		return m, nil
	case doneMsg:
		m.done = true
		m.exitCode = msg.ExitCode
		if msg.ExitCode != 0 || m.cancelled {
			return m, tea.Quit
		}
		m.status = "finished"
		m.pct = 100
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c":
			m.cancelled = true
			m.dl.Kill()
			if m.done {
				return m, tea.Quit
			}
			return m, nil // wait for doneMsg so yt-dlp is reaped
		case "esc", "enter":
			// enter/esc on the last state: apply and leave once the download is
			// done. While downloading, enter advances fields and esc cancels.
			if m.done {
				return m, tea.Quit
			}
			if msg.String() == "esc" {
				m.cancelled = true
				m.dl.Kill()
				return m, nil
			}
			m.setFocus(m.focus + 1)
			return m, nil
		case "tab", "down":
			m.setFocus(m.focus + 1)
			return m, nil
		case "shift+tab", "up":
			m.setFocus(m.focus - 1)
			return m, nil
		case "ctrl+p":
			if paused, err := m.dl.TogglePause(); err == nil {
				m.paused = paused
			}
			return m, nil
		case "ctrl+o":
			if p := m.dl.FindPart(m.pf, m.finalReal); p != "" {
				_ = m.dl.OpenMPV(p)
			}
			return m, nil
		}
		// rating row: not a text input — digits and arrows set it directly
		if m.focus == fRating {
			switch msg.String() {
			case "0", "1", "2", "3", "4", "5":
				m.rating = int(msg.String()[0] - '0')
			case "left", "h":
				if m.rating > 0 {
					m.rating--
				}
			case "right", "l":
				if m.rating < 5 {
					m.rating++
				}
			}
			return m, nil
		}
		var cmd tea.Cmd
		m.inputs[m.focus], cmd = m.inputs[m.focus].Update(msg)
		return m, cmd
	}
	// blink etc. for the focused input
	if m.focus <= fTags {
		var cmd tea.Cmd
		m.inputs[m.focus], cmd = m.inputs[m.focus].Update(msg)
		return m, cmd
	}
	return m, nil
}

func starsColored(n int) string {
	var b strings.Builder
	for i := 1; i <= 5; i++ {
		if i <= n {
			b.WriteString(stStar.Render("★"))
		} else {
			b.WriteString(stDim.Render("☆"))
		}
		if i < 5 {
			b.WriteByte(' ')
		}
	}
	return b.String()
}

func (m model) bar(width int) string {
	if width < 8 {
		width = 8
	}
	filled := width * m.pct / 100
	if filled > width {
		filled = width
	}
	return stNeon.Render(strings.Repeat("█", filled)) +
		stDim.Render(strings.Repeat("░", width-filled))
}

func clip(s string, n int) string {
	r := []rune(s)
	if n > 0 && len(r) > n {
		return string(r[:n-1]) + "…"
	}
	return s
}

func (m model) View() string {
	var b strings.Builder
	textw := m.width - 4
	if textw < 20 {
		textw = 20
	}
	barw := m.width - 10
	if barw > 46 {
		barw = 46
	}

	b.WriteString("\n  " + stBold.Render("ytform") + " " + stDim.Render(clip(m.url, textw-8)) + "\n\n")

	if m.thumb != "" {
		for _, l := range strings.Split(m.thumb, "\n") {
			b.WriteString("  " + l + "\n")
		}
		b.WriteString("\n")
	}

	state := stAccent.Render(m.status)
	if m.paused {
		state = stWarn.Render("paused")
	} else if m.status == "finished" {
		state = stOK.Render("finished")
	}
	rule := strings.Repeat("─", barw+2)
	b.WriteString("  " + stBorder.Render("╭"+rule+"╮") + "\n")
	b.WriteString("  " + stBorder.Render("│") + " " + m.bar(barw) + " " + stBorder.Render("│") + "\n")
	b.WriteString("  " + stBorder.Render("╰"+rule+"╯") + "\n")
	b.WriteString(fmt.Sprintf("   %s  %s  %s  %s  %s\n",
		stBold.Render(fmt.Sprintf("%d%%", m.pct)),
		stDim.Render(m.dlStr+" / "+m.totStr),
		stAccent.Render(m.spdStr),
		stDim.Render("ETA "+m.eta),
		state))
	if m.phase != "" {
		b.WriteString("   " + stDim.Render(clip(m.phase, textw)) + "\n")
	}
	b.WriteString("\n")

	for i := 0; i < fCount; i++ {
		cursor := "  "
		if i == m.focus {
			cursor = stAccent.Render("▸ ")
		}
		label := stLabel.Render(fmt.Sprintf("%-8s", fieldLabels[i]))
		if i == fRating {
			b.WriteString(fmt.Sprintf("  %s%s %s  %s\n", cursor, label,
				starsColored(m.rating), stDim.Render(fmt.Sprintf("%d/5", m.rating))))
		} else {
			b.WriteString(fmt.Sprintf("  %s%s %s\n", cursor, label, m.inputs[i].View()))
		}
	}

	b.WriteString("\n  " + stLabel.Render("file    ") + " " +
		stFile.Render(clip(m.meta().Stem()+"."+m.pf.Ext, textw-8)) + "\n\n")

	if m.done {
		b.WriteString("  " + stOK.Render("done") + stDim.Render(" — ") +
			stBold.Render("enter") + stDim.Render(" apply & exit\n"))
	} else {
		b.WriteString("  " + stDim.Render("tab/↑↓ fields · ctrl+o mpv · ctrl+p pause · esc cancel") + "\n")
	}
	return b.String()
}
