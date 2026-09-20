// iospeed — a tiny disk I/O benchmark with a realtime TUI.
//
// Writes a test file of a given size (default 1G), then reads it back,
// showing live throughput while each phase runs. On macOS the page cache
// is bypassed with F_NOCACHE so the numbers reflect the disk, not RAM.
package main

import (
	"crypto/rand"
	"flag"
	"fmt"
	"io"
	"os"
	"os/signal"
	"path/filepath"
	"strconv"
	"strings"
	"sync/atomic"
	"syscall"
	"time"

	"github.com/charmbracelet/bubbles/progress"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// ---------------------------------------------------------------------------
// benchmark engine
// ---------------------------------------------------------------------------

type phase int

const (
	phaseWrite phase = iota
	phaseRead
	phaseDone
)

type bench struct {
	path        string
	size        int64
	block       int64
	cacheBypass bool
	keep        bool

	written  atomic.Int64
	readback atomic.Int64
	cancel   atomic.Bool
}

type phaseDoneMsg struct {
	phase    phase
	duration time.Duration
}

type benchDoneMsg struct{}
type benchAbortedMsg struct{}
type benchErrMsg struct{ err error }

func (b *bench) fail(ch chan<- tea.Msg, err error) {
	ch <- benchErrMsg{err}
}

// run executes the write and read phases sequentially, reporting milestones
// over ch. Progress is exposed through the atomic counters.
func (b *bench) run(ch chan<- tea.Msg) {
	defer func() {
		if !b.keep {
			os.Remove(b.path)
		}
	}()

	buf := make([]byte, b.block)
	if _, err := rand.Read(buf); err != nil {
		b.fail(ch, err)
		return
	}

	// --- write phase ---
	f, err := os.OpenFile(b.path, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, 0o644)
	if err != nil {
		b.fail(ch, err)
		return
	}
	if b.cacheBypass {
		disableCache(f) // best effort; numbers are still valid without it
	}
	start := time.Now()
	for b.written.Load() < b.size {
		if b.cancel.Load() {
			f.Close()
			ch <- benchAbortedMsg{}
			return
		}
		n := min(b.block, b.size-b.written.Load())
		if _, err := f.Write(buf[:n]); err != nil {
			f.Close()
			b.fail(ch, err)
			return
		}
		b.written.Add(n)
	}
	if err := f.Sync(); err != nil {
		f.Close()
		b.fail(ch, err)
		return
	}
	f.Close()
	ch <- phaseDoneMsg{phaseWrite, time.Since(start)}

	// --- read phase ---
	f, err = os.Open(b.path)
	if err != nil {
		b.fail(ch, err)
		return
	}
	if b.cacheBypass {
		disableCache(f)
	}
	start = time.Now()
	for b.readback.Load() < b.size {
		if b.cancel.Load() {
			f.Close()
			ch <- benchAbortedMsg{}
			return
		}
		n := min(b.block, b.size-b.readback.Load())
		if _, err := io.ReadFull(f, buf[:n]); err != nil {
			f.Close()
			b.fail(ch, err)
			return
		}
		b.readback.Add(n)
	}
	f.Close()
	ch <- phaseDoneMsg{phaseRead, time.Since(start)}
	ch <- benchDoneMsg{}
}

// ---------------------------------------------------------------------------
// realtime speed over a sliding window
// ---------------------------------------------------------------------------

type sample struct {
	t time.Time
	n int64
}

type speedometer struct{ samples []sample }

const speedWindow = time.Second

func (s *speedometer) add(n int64) {
	now := time.Now()
	s.samples = append(s.samples, sample{now, n})
	cutoff := now.Add(-speedWindow)
	for len(s.samples) > 2 && s.samples[0].t.Before(cutoff) {
		s.samples = s.samples[1:]
	}
}

func (s *speedometer) reset() { s.samples = s.samples[:0] }

// current returns bytes/sec over the window, or 0 if not enough data yet.
func (s *speedometer) current() float64 {
	if len(s.samples) < 2 {
		return 0
	}
	first, last := s.samples[0], s.samples[len(s.samples)-1]
	dt := last.t.Sub(first.t).Seconds()
	if dt <= 0 {
		return 0
	}
	return float64(last.n-first.n) / dt
}

// ---------------------------------------------------------------------------
// TUI
// ---------------------------------------------------------------------------

var (
	violet  = lipgloss.Color("#7D56F4")
	pink    = lipgloss.Color("#EE6FF8")
	teal    = lipgloss.Color("#04B575")
	errRed  = lipgloss.Color("#FF5F87")
	dimmed  = lipgloss.AdaptiveColor{Light: "#A49FA5", Dark: "#777777"}
	bright  = lipgloss.AdaptiveColor{Light: "#1A1A1A", Dark: "#DDDDDD"}

	titleStyle  = lipgloss.NewStyle().Bold(true).Foreground(violet)
	subtleStyle = lipgloss.NewStyle().Foreground(dimmed)
	labelStyle  = lipgloss.NewStyle().Bold(true).Foreground(bright).Width(7)
	speedStyle  = lipgloss.NewStyle().Bold(true).Foreground(pink)
	doneStyle   = lipgloss.NewStyle().Bold(true).Foreground(teal)
	errStyle    = lipgloss.NewStyle().Bold(true).Foreground(errRed)
	boxStyle    = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(violet).
			Padding(0, 2)
)

type tickMsg time.Time

func tick() tea.Cmd {
	return tea.Tick(80*time.Millisecond, func(t time.Time) tea.Msg { return tickMsg(t) })
}

func listen(ch <-chan tea.Msg) tea.Cmd {
	return func() tea.Msg { return <-ch }
}

type model struct {
	b  *bench
	ch chan tea.Msg

	phase     phase
	writeBar  progress.Model
	readBar   progress.Model
	speed     speedometer
	writeDur  time.Duration
	readDur   time.Duration
	width     int
	quitting  bool
	aborted   bool
	err       error
}

func newModel(b *bench, ch chan tea.Msg) model {
	mk := func(from, to string) progress.Model {
		p := progress.New(progress.WithGradient(from, to), progress.WithoutPercentage())
		p.Width = 40
		return p
	}
	return model{
		b:        b,
		ch:       ch,
		writeBar: mk("#5A56E0", "#EE6FF8"),
		readBar:  mk("#0496FF", "#04B575"),
	}
}

func (m model) Init() tea.Cmd {
	return tea.Batch(listen(m.ch), tick())
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {

	case tea.KeyMsg:
		switch msg.String() {
		case "q", "esc", "ctrl+c":
			if m.phase == phaseDone || m.err != nil {
				return m, tea.Quit
			}
			// Ask the worker to stop, then wait for benchAbortedMsg so the
			// test file is cleaned up before we exit.
			m.quitting = true
			m.b.cancel.Store(true)
			return m, nil
		}

	case tea.WindowSizeMsg:
		m.width = msg.Width
		barWidth := max(min(msg.Width-34, 60), 10)
		m.writeBar.Width = barWidth
		m.readBar.Width = barWidth

	case tickMsg:
		if m.phase == phaseDone || m.err != nil {
			return m, nil
		}
		switch m.phase {
		case phaseWrite:
			m.speed.add(m.b.written.Load())
		case phaseRead:
			m.speed.add(m.b.readback.Load())
		}
		return m, tick()

	case phaseDoneMsg:
		m.speed.reset()
		switch msg.phase {
		case phaseWrite:
			m.writeDur = msg.duration
			m.phase = phaseRead
		case phaseRead:
			m.readDur = msg.duration
		}
		return m, listen(m.ch)

	case benchDoneMsg:
		m.phase = phaseDone
		return m, tea.Quit

	case benchAbortedMsg:
		m.aborted = true
		return m, tea.Quit

	case benchErrMsg:
		m.err = msg.err
		return m, tea.Quit
	}

	return m, nil
}

func (m model) phaseRow(p phase, bar progress.Model, label string, done int64, dur time.Duration) string {
	pct := float64(done) / float64(m.b.size)

	var status string
	switch {
	case p < m.phase || (p == phaseRead && m.readDur > 0):
		status = doneStyle.Render(fmtSpeed(float64(m.b.size)/dur.Seconds())) +
			subtleStyle.Render("  "+fmtDur(dur))
		pct = 1
	case p > m.phase:
		status = subtleStyle.Render("waiting")
		pct = 0
	default:
		cur := m.speed.current()
		if cur <= 0 {
			status = subtleStyle.Render("…")
		} else {
			remaining := float64(m.b.size-done) / cur
			status = speedStyle.Render(fmtSpeed(cur)) +
				subtleStyle.Render(fmt.Sprintf("  eta %s", fmtDur(time.Duration(remaining*float64(time.Second)))))
		}
	}

	return fmt.Sprintf("  %s %s %s  %s",
		labelStyle.Render(label),
		bar.ViewAs(pct),
		subtleStyle.Render(fmt.Sprintf("%3.0f%%", pct*100)),
		status)
}

func (m model) View() string {
	var s strings.Builder

	s.WriteString("\n  " + titleStyle.Render("iospeed") + subtleStyle.Render("  ·  disk I/O benchmark") + "\n\n")
	s.WriteString("  " + subtleStyle.Render(fmt.Sprintf("target %s   size %s   block %s   cache bypass %s",
		filepath.Dir(m.b.path), fmtBytes(m.b.size), fmtBytes(m.b.block), onOff(m.b.cacheBypass))) + "\n\n")

	s.WriteString(m.phaseRow(phaseWrite, m.writeBar, "write", m.b.written.Load(), m.writeDur) + "\n")
	s.WriteString(m.phaseRow(phaseRead, m.readBar, "read", m.b.readback.Load(), m.readDur) + "\n")

	switch {
	case m.err != nil:
		s.WriteString("\n  " + errStyle.Render("error: "+m.err.Error()) + "\n")
	case m.aborted:
		s.WriteString("\n  " + subtleStyle.Render("aborted — test file removed") + "\n")
	case m.phase == phaseDone:
		summary := fmt.Sprintf("%s  %s\n%s  %s",
			labelStyle.Render("write"), doneStyle.Render(fmtSpeed(float64(m.b.size)/m.writeDur.Seconds())),
			labelStyle.Render("read"), doneStyle.Render(fmtSpeed(float64(m.b.size)/m.readDur.Seconds())))
		s.WriteString("\n" + lipgloss.NewStyle().MarginLeft(2).Render(boxStyle.Render(summary)) + "\n")
	case m.quitting:
		s.WriteString("\n  " + subtleStyle.Render("stopping…") + "\n")
	default:
		s.WriteString("\n  " + subtleStyle.Render("q to quit") + "\n")
	}

	return s.String()
}

// ---------------------------------------------------------------------------
// plain mode (no TTY)
// ---------------------------------------------------------------------------

func runPlain(b *bench) error {
	ch := make(chan tea.Msg, 4)
	go b.run(ch)
	for msg := range ch {
		switch msg := msg.(type) {
		case phaseDoneMsg:
			name := "write"
			if msg.phase == phaseRead {
				name = "read"
			}
			fmt.Printf("%-6s %s  (%s, %s)\n",
				name, fmtSpeed(float64(b.size)/msg.duration.Seconds()), fmtBytes(b.size), fmtDur(msg.duration))
		case benchDoneMsg:
			return nil
		case benchAbortedMsg:
			return fmt.Errorf("aborted")
		case benchErrMsg:
			return msg.err
		}
	}
	return nil
}

// ---------------------------------------------------------------------------
// formatting helpers
// ---------------------------------------------------------------------------

func onOff(b bool) string {
	if b {
		return "on"
	}
	return "off"
}

func fmtBytes(n int64) string {
	const unit = 1024
	switch {
	case n >= unit*unit*unit:
		return trimZero(float64(n)/(unit*unit*unit)) + " GiB"
	case n >= unit*unit:
		return trimZero(float64(n)/(unit*unit)) + " MiB"
	case n >= unit:
		return trimZero(float64(n)/unit) + " KiB"
	default:
		return fmt.Sprintf("%d B", n)
	}
}

// fmtSpeed uses decimal units (MB/s, GB/s) like most disk benchmarks.
func fmtSpeed(bps float64) string {
	switch {
	case bps >= 1e9:
		return fmt.Sprintf("%.2f GB/s", bps/1e9)
	case bps >= 1e6:
		return fmt.Sprintf("%.0f MB/s", bps/1e6)
	default:
		return fmt.Sprintf("%.0f KB/s", bps/1e3)
	}
}

func fmtDur(d time.Duration) string {
	switch {
	case d >= time.Minute:
		return fmt.Sprintf("%dm%02ds", int(d.Minutes()), int(d.Seconds())%60)
	case d >= 10*time.Second:
		return fmt.Sprintf("%.0fs", d.Seconds())
	default:
		return fmt.Sprintf("%.1fs", d.Seconds())
	}
}

func trimZero(f float64) string {
	return strings.TrimSuffix(strconv.FormatFloat(f, 'f', 1, 64), ".0")
}

// parseSize accepts forms like "1G", "512M", "1.5GiB", "100MB", "4096".
func parseSize(s string) (int64, error) {
	s = strings.TrimSpace(strings.ToUpper(s))
	num := strings.TrimRight(s, "KMGTIB")
	suffix := s[len(num):]

	mult := int64(1)
	switch strings.TrimSuffix(strings.TrimSuffix(suffix, "B"), "I") {
	case "":
	case "K":
		mult = 1 << 10
	case "M":
		mult = 1 << 20
	case "G":
		mult = 1 << 30
	case "T":
		mult = 1 << 40
	default:
		return 0, fmt.Errorf("unknown size suffix %q", suffix)
	}

	f, err := strconv.ParseFloat(num, 64)
	if err != nil || f <= 0 {
		return 0, fmt.Errorf("invalid size %q", s)
	}
	return int64(f * float64(mult)), nil
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

func main() {
	sizeFlag := flag.String("size", "1G", "test file size (e.g. 512M, 2G)")
	blockFlag := flag.String("block", "4M", "I/O block size")
	keep := flag.Bool("keep", false, "keep the test file afterwards")
	cached := flag.Bool("cached", false, "allow the OS page cache (measures cache, not disk)")
	flag.Usage = func() {
		fmt.Fprintf(os.Stderr, "usage: iospeed [flags] [dir]\n\nBenchmarks write then read speed of a test file in dir (default: current dir).\n\n")
		flag.PrintDefaults()
	}
	flag.Parse()

	dir := "."
	if flag.NArg() > 0 {
		dir = flag.Arg(0)
	}
	absDir, err := filepath.Abs(dir)
	if err == nil {
		dir = absDir
	}
	if st, err := os.Stat(dir); err != nil || !st.IsDir() {
		fmt.Fprintf(os.Stderr, "iospeed: %s is not a directory\n", dir)
		os.Exit(1)
	}

	size, err := parseSize(*sizeFlag)
	if err != nil {
		fmt.Fprintln(os.Stderr, "iospeed:", err)
		os.Exit(1)
	}
	block, err := parseSize(*blockFlag)
	if err != nil {
		fmt.Fprintln(os.Stderr, "iospeed:", err)
		os.Exit(1)
	}

	b := &bench{
		path:        filepath.Join(dir, fmt.Sprintf(".iospeed-%d.tmp", os.Getpid())),
		size:        size,
		block:       min(block, size),
		cacheBypass: !*cached,
		keep:        *keep,
	}

	// Clean up the test file even on SIGINT/SIGTERM in plain mode
	// (the TUI handles ctrl+c itself).
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	go func() {
		<-sig
		b.cancel.Store(true)
	}()

	if st, err := os.Stdout.Stat(); err == nil && st.Mode()&os.ModeCharDevice == 0 {
		if err := runPlain(b); err != nil {
			fmt.Fprintln(os.Stderr, "iospeed:", err)
			os.Exit(1)
		}
		return
	}

	ch := make(chan tea.Msg, 4)
	go b.run(ch)
	if _, err := tea.NewProgram(newModel(b, ch)).Run(); err != nil {
		fmt.Fprintln(os.Stderr, "iospeed:", err)
		os.Exit(1)
	}
}
