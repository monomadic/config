// rclone-tower-ui presents the guarded shell workflow; it never runs sync itself.
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"math"
	"os"
	"os/exec"
	"os/signal"
	"strings"
	"sync"
	"syscall"
	"time"
	"unicode"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
)

type transfer struct {
	Name       string  `json:"name"`
	Size       int64   `json:"size"`
	Bytes      int64   `json:"bytes"`
	Speed      float64 `json:"speed"`
	Percentage int     `json:"percentage"`
}
type stats struct {
	Renames        int64      `json:"renames"`
	DeletedDirs    int64      `json:"deletedDirs"`
	Bytes          int64      `json:"bytes"`
	TotalBytes     int64      `json:"totalBytes"`
	Speed          float64    `json:"speed"`
	Checks         int64      `json:"checks"`
	TotalChecks    int64      `json:"totalChecks"`
	Transfers      int64      `json:"transfers"`
	TotalTransfers int64      `json:"totalTransfers"`
	Errors         int64      `json:"errors"`
	Deletes        int64      `json:"deletes"`
	Listed         int64      `json:"listed"`
	ETA            *float64   `json:"eta"`
	Transferring   []transfer `json:"transferring"`
	Checking       []string   `json:"checking"`
}
type record struct {
	Level  string `json:"level"`
	Msg    string `json:"msg"`
	Object string `json:"object"`
	Stats  *stats `json:"stats"`
}
type lineMsg string
type doneMsg int
type tickMsg time.Time
type diskMsg struct{ free, total uint64 }

var (
	bright    = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#1EE6FF"))
	pink      = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#FF2EC0"))
	dim       = lipgloss.NewStyle().Foreground(lipgloss.Color("#9AA4B2"))
	okStyle   = lipgloss.NewStyle().Foreground(lipgloss.Color("#3BE38B"))
	warnStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFC24B"))
)

// Never allow filenames or subprocess messages to inject terminal controls.
func clean(s string) string {
	return strings.Map(func(r rune) rune {
		if unicode.IsControl(r) {
			return ' '
		}
		return r
	}, ansi.Strip(s))
}
func human(n float64) string {
	units := []string{"B", "KiB", "MiB", "GiB", "TiB"}
	i := 0
	for n >= 1024 && i < len(units)-1 {
		n /= 1024
		i++
	}
	return fmt.Sprintf("%.1f %s", n, units[i])
}
func bar(ratio float64, width int) string {
	width = max(8, width)
	n := int(math.Max(0, math.Min(1, ratio)) * float64(width))
	return pink.Render(strings.Repeat("━", n)) + dim.Render(strings.Repeat("─", width-n))
}
func tick() tea.Cmd { return tea.Tick(time.Second, func(t time.Time) tea.Msg { return tickMsg(t) }) }
func diskSpace() tea.Msg {
	var st syscall.Statfs_t
	if syscall.Statfs("/Volumes/Tower Backup", &st) != nil {
		return diskMsg{}
	}
	return diskMsg{uint64(st.Bavail) * uint64(st.Bsize), uint64(st.Blocks) * uint64(st.Bsize)}
}

// Logs are display data only. Rotation never affects synchronization correctness.
type logReader struct {
	path    string
	info    os.FileInfo
	offset  int64
	pending string
}

func (r *logReader) read() []string {
	if r.path == "" {
		return nil
	}
	f, err := os.Open(r.path)
	if err != nil {
		return nil
	}
	defer f.Close()
	st, err := f.Stat()
	if err != nil {
		return nil
	}
	if r.info == nil || !os.SameFile(r.info, st) || st.Size() < r.offset {
		r.offset = 0
		r.pending = ""
	}
	r.info = st
	if _, err = f.Seek(r.offset, io.SeekStart); err != nil {
		return nil
	}
	b, err := io.ReadAll(io.LimitReader(f, 4<<20))
	if err != nil {
		return nil
	}
	r.offset += int64(len(b))
	parts := strings.Split(r.pending+string(b), "\n")
	r.pending = parts[len(parts)-1]
	return parts[:len(parts)-1]
}

type model struct {
	w, h           int
	phase          string
	started        time.Time
	finished       time.Time
	stats          stats
	logs           []string
	warnings       int
	lastWarning    string
	done, stopping bool
	code           int
	stop           func()
	reader         *logReader
	free, total    uint64
	mode           string
}

func (m model) Init() tea.Cmd { return tea.Batch(tick(), diskSpace) }
func (m *model) add(s string) {
	m.logs = append(m.logs, clean(s))
	if len(m.logs) > 6 {
		m.logs = m.logs[len(m.logs)-6:]
	}
}
func (m *model) consume(s string) {
	var r record
	if json.Unmarshal([]byte(s), &r) != nil {
		if strings.TrimSpace(s) != "" {
			m.add(s)
		}
		return
	}
	if r.Stats != nil {
		m.stats = *r.Stats
		return
	}
	if r.Level == "warning" || r.Level == "error" || (r.Level == "notice" && !strings.Contains(r.Msg, "Config file")) {
		m.warnings++
		m.lastWarning = clean(r.Object + " " + r.Msg)
	}
	if strings.Contains(r.Msg, "Updated directory metadata") {
		return
	}
	if r.Object != "" {
		m.add(r.Object + ": " + r.Msg)
	} else {
		m.add(r.Msg)
	}
}
func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch v := msg.(type) {
	case tea.WindowSizeMsg:
		m.w, m.h = v.Width, v.Height
	case tea.KeyMsg:
		if v.String() == "q" || v.String() == "ctrl+c" || v.String() == "esc" {
			if m.done {
				return m, tea.Quit
			}
			if !m.stopping {
				m.stopping = true
				m.phase = "Stopping safely — waiting for the active operation"
				m.stop()
			}
		}
	case lineMsg:
		s := string(v)
		if strings.HasPrefix(s, "Stage: ") {
			if !m.stopping {
				m.phase = clean(strings.TrimPrefix(s, "Stage: "))
			}
		} else if strings.HasPrefix(s, "Log: ") {
			m.reader.path = strings.TrimPrefix(s, "Log: ")
		} else {
			m.add(s)
		}
	case tickMsg:
		for _, s := range m.reader.read() {
			m.consume(s)
		}
		return m, tea.Batch(tick(), diskSpace)
	case diskMsg:
		m.free, m.total = v.free, v.total
	case doneMsg:
		for _, s := range m.reader.read() {
			m.consume(s)
		}
		m.done = true
		m.code = int(v)
		m.finished = time.Now()
		if m.code == 0 {
			m.phase = "Sync complete"
			if m.mode == "--check-only" {
				m.phase = "Filesystem checks passed"
			}
			if m.mode == "--dry-run" {
				m.phase = "Preview complete — no files changed"
			}
		} else {
			m.phase = fmt.Sprintf("Stopped — exit %d · review the messages below", m.code)
		}
	}
	return m, nil
}
func (m model) View() string {
	w := min(96, max(20, m.w-6))
	bw := min(60, max(8, w-12))
	var b strings.Builder
	b.WriteString(pink.Render("TOWER") + bright.Render("  →  TOWER BACKUP") + "\n")
	b.WriteString(dim.Render("One-way mirror · recoverable history · size + time rename matching") + "\n\n")
	elapsed := time.Since(m.started)
	if m.done {
		elapsed = m.finished.Sub(m.started)
	}
	b.WriteString(bright.Render(m.phase) + "\n" + dim.Render("Elapsed "+elapsed.Round(time.Second).String()) + "\n\n")
	if m.reader.path != "" {
		if len(m.stats.Transferring) > 0 && !m.done {
			f := m.stats.Transferring[0]
			b.WriteString(ansi.Truncate(clean(f.Name), w, "…") + "\n")
			b.WriteString(fmt.Sprintf("%3d%% %s\n", f.Percentage, bar(float64(f.Percentage)/100, bw)))
			b.WriteString(fmt.Sprintf("%s / %s · %s/s\n", human(float64(f.Bytes)), human(float64(f.Size)), human(f.Speed)))
		} else if len(m.stats.Checking) > 0 && !m.done {
			b.WriteString("Inspecting " + ansi.Truncate(clean(m.stats.Checking[0]), max(8, w-11), "…") + "\n")
		} else {
			b.WriteString(dim.Render("Scanning / finishing filesystem operations") + "\n")
		}
		eta := "calculating"
		if m.stats.ETA != nil {
			eta = (time.Duration(*m.stats.ETA) * time.Second).String()
		}
		b.WriteString(fmt.Sprintf("\nTransferred %s / %s · %s/s · ETA %s\n", human(float64(m.stats.Bytes)), human(float64(m.stats.TotalBytes)), human(m.stats.Speed), eta))
		b.WriteString(fmt.Sprintf("Listed %d · Checked %d/%d · Transfers %d/%d · Removed %d · Errors %d\n", m.stats.Listed, m.stats.Checks, m.stats.TotalChecks, m.stats.Transfers, m.stats.TotalTransfers, m.stats.Deletes, m.stats.Errors))
		b.WriteString(dim.Render("Totals grow during scanning. Wait for the completed result.") + "\n")
		b.WriteString(fmt.Sprintf("Renamed %d · Empty folders removed %d\n", m.stats.Renames, m.stats.DeletedDirs))
	}
	if m.total > 0 {
		b.WriteString("\n" + bar(1-float64(m.free)/float64(m.total), bw) + "\n" + fmt.Sprintf("Backup disk · %s free / %s total\n", human(float64(m.free)), human(float64(m.total))))
	}
	if m.warnings > 0 {
		b.WriteString("\n" + warnStyle.Render(fmt.Sprintf("%d warnings / errors · inspect the log for skipped items", m.warnings)) + "\n" + ansi.Truncate(m.lastWarning, w, "…") + "\n")
	}
	b.WriteString("\n" + dim.Render("RECENT ACTIVITY") + "\n")
	budget := max(0, m.h-2-strings.Count(b.String(), "\n")-7)
	logs := m.logs[max(0, len(m.logs)-budget):]
	for _, s := range logs {
		b.WriteString(ansi.Truncate(s, w, "…") + "\n")
	}
	if m.reader.path != "" {
		b.WriteString("\n" + dim.Render("Log: "+m.reader.path) + "\n")
	}
	footer := "q / esc / ctrl+c · stop safely"
	if m.done {
		footer = "q / esc · close"
		if m.code == 0 {
			b.WriteString("\n" + okStyle.Render(m.phase) + "\n")
		}
	}
	b.WriteString("\n" + dim.Render(footer))
	lines := strings.Split(b.String(), "\n")
	for i := range lines {
		lines[i] = ansi.Truncate(lines[i], w, "…")
	}
	if m.h > 6 && len(lines) > m.h-2 {
		lines = append(lines[:m.h-6], lines[len(lines)-4:]...)
	}
	return lipgloss.NewStyle().Padding(1, 2).Render(strings.Join(lines, "\n"))
}

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "Run rclone-tower-safe to open this dashboard.")
		os.Exit(2)
	}
	cmd := exec.Command("/bin/zsh", append([]string{"-f", os.Args[1]}, os.Args[2:]...)...)
	cmd.Env = append(os.Environ(), "RCLONE_TOWER_DASHBOARD=1")
	// No terminal stdin: automatic pruning stays disabled. Use --prune separately.
	r, w, err := os.Pipe()
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	cmd.Stdout = w
	cmd.Stderr = w
	if err = cmd.Start(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	w.Close()
	var once sync.Once
	stop := func() { once.Do(func() { _ = cmd.Process.Signal(syscall.SIGTERM) }) }
	mode := "sync"
	if len(os.Args) > 2 {
		mode = os.Args[2]
	}
	p := tea.NewProgram(model{phase: "Checking disk identities", started: time.Now(), stop: stop, reader: &logReader{}, mode: mode}, tea.WithAltScreen())
	signals := make(chan os.Signal, 1)
	signal.Notify(signals, syscall.SIGTERM, syscall.SIGHUP)
	go func() {
		for range signals {
			stop()
		}
	}()
	complete := make(chan int, 1)
	go func() {
		sc := bufio.NewScanner(r)
		sc.Buffer(make([]byte, 4096), 1<<20)
		for sc.Scan() {
			p.Send(lineMsg(sc.Text()))
		}
		r.Close()
		if sc.Err() != nil {
			stop()
		}
		err := cmd.Wait()
		code := 0
		if err != nil {
			code = 1
			if e, ok := err.(*exec.ExitError); ok {
				code = e.ExitCode()
				if code < 0 {
					code = 130
				}
			}
		}
		complete <- code
		p.Send(doneMsg(code))
	}()
	_, err = p.Run()
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	stop()
	code := <-complete
	if err != nil && code == 0 {
		code = 1
	}
	fmt.Printf("Tower backup finished with exit %d.\n", code)
	os.Exit(code)
}
