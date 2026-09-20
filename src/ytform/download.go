package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"sync"
	"syscall"

	tea "github.com/charmbracelet/bubbletea"
)

// messages the download goroutine sends into the UI
type progressMsg struct {
	Pct               int
	DL, Tot, Spd, Eta string
	Status            string
}
type finalPathMsg string // after_move:FINALPATH — the real output path
type logMsg string       // any other yt-dlp line (phase message / error ring)
type doneMsg struct{ ExitCode int }
type thumbMsg string // rendered ANSI thumbnail

// Downloader owns the yt-dlp child so the UI can pause it and find its
// growing .part file.
type Downloader struct {
	mu       sync.Mutex
	cmd      *exec.Cmd
	paused   bool
	destPath string   // from "[download] Destination: …" (the temp: path)
	logRing  []string // recent yt-dlp lines, for error reporting
}

var ansiRE = regexp.MustCompile(`\x1b\[[0-9;]*[a-zA-Z]`)

// Run starts yt-dlp and pumps its output into the program as messages.
// Call from a goroutine after p is constructed.
func (d *Downloader) Run(p *tea.Program, passthru []string, url string) {
	args := []string{
		"--no-playlist", "--newline", "--no-download-archive", "--continue",
		"--progress-template",
		"download:PROGRESS|%(progress._percent_str)s|%(progress._downloaded_bytes_str)s|%(progress._total_bytes_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(progress.status)s|%(progress.downloaded_bytes)d",
		"--print", "after_move:FINALPATH|%(filepath)s",
	}
	args = append(args, passthru...)
	args = append(args, url)

	cmd := exec.Command("yt-dlp", args...)
	out, err := cmd.StdoutPipe()
	if err != nil {
		p.Send(logMsg(err.Error()))
		p.Send(doneMsg{ExitCode: 1})
		return
	}
	cmd.Stderr = cmd.Stdout
	if err := cmd.Start(); err != nil {
		p.Send(logMsg(err.Error()))
		p.Send(doneMsg{ExitCode: 1})
		return
	}
	d.mu.Lock()
	d.cmd = cmd
	d.mu.Unlock()

	sc := bufio.NewScanner(out)
	sc.Buffer(make([]byte, 0, 64*1024), 1024*1024)
	for sc.Scan() {
		d.handleLine(p, sc.Text())
	}
	ec := 0
	if err := cmd.Wait(); err != nil {
		ec = 1
		if ee, ok := err.(*exec.ExitError); ok {
			ec = ee.ExitCode()
		}
	}
	p.Send(doneMsg{ExitCode: ec})
}

func (d *Downloader) handleLine(p *tea.Program, line string) {
	if rest, ok := strings.CutPrefix(line, "PROGRESS|"); ok {
		f := strings.Split(rest, "|")
		for len(f) < 7 {
			f = append(f, "")
		}
		m := progressMsg{
			DL: strings.TrimSpace(f[1]), Tot: strings.TrimSpace(f[2]),
			Spd: strings.TrimSpace(f[3]), Eta: strings.TrimSpace(f[4]),
			Status: strings.TrimSpace(f[5]),
		}
		nf := regexp.MustCompile(`[^0-9.]`).ReplaceAllString(f[0], "")
		if v, err := strconv.ParseFloat(nf, 64); err == nil {
			m.Pct = int(v)
		}
		if m.Status == "finished" {
			m.Pct = 100
		}
		// HLS/live streams report no total up front, but we do get bytes-so-far
		// and a percentage — back out an estimate and mark it "~".
		switch m.Tot {
		case "N/A", "NA", "Unknown", "unknown", "-", "":
			if db, err := strconv.ParseInt(strings.TrimSpace(f[6]), 10, 64); err == nil && db > 0 && m.Pct > 0 {
				m.Tot = "~" + humanBytes(db*100/int64(m.Pct))
			}
		}
		p.Send(m)
		return
	}
	if rest, ok := strings.CutPrefix(line, "FINALPATH|"); ok {
		p.Send(finalPathMsg(rest))
		return
	}
	clean := strings.TrimSpace(ansiRE.ReplaceAllString(line, ""))
	if clean == "" {
		return
	}
	// "[download] Destination: <path>" is where the growing .part actually
	// lives (the yt-dlp temp: path) — mpv streaming needs it as a fallback.
	if _, dest, ok := strings.Cut(clean, "Destination: "); ok {
		d.mu.Lock()
		d.destPath = dest
		d.mu.Unlock()
	}
	d.mu.Lock()
	d.logRing = append(d.logRing, clean)
	if len(d.logRing) > 40 {
		d.logRing = d.logRing[1:]
	}
	d.mu.Unlock()
	p.Send(logMsg(clean))
}

// TogglePause SIGSTOP/SIGCONTs the yt-dlp child. Returns the new state.
func (d *Downloader) TogglePause() (bool, error) {
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.cmd == nil || d.cmd.Process == nil {
		return false, fmt.Errorf("download not running")
	}
	sig := syscall.SIGSTOP
	if d.paused {
		sig = syscall.SIGCONT
	}
	if err := d.cmd.Process.Signal(sig); err != nil {
		return d.paused, err
	}
	d.paused = !d.paused
	return d.paused, nil
}

// Kill resumes (a stopped process can't be signalled to die) then kills.
func (d *Downloader) Kill() {
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.cmd != nil && d.cmd.Process != nil {
		_ = d.cmd.Process.Signal(syscall.SIGCONT)
		_ = d.cmd.Process.Kill()
	}
}

func (d *Downloader) LogTail(n int) []string {
	d.mu.Lock()
	defer d.mu.Unlock()
	if len(d.logRing) > n {
		return append([]string(nil), d.logRing[len(d.logRing)-n:]...)
	}
	return append([]string(nil), d.logRing...)
}

// FindPart locates the growing partial. The one authoritative source is the
// running yt-dlp itself: whatever .part fd IT holds open is the real file
// (with --paths temp:… it lives in a temp tree, not the final dir). lsof on
// our own child gives exactly that; the Destination line and dir globs are
// fallbacks.
func (d *Downloader) FindPart(pf *PreflightInfo, finalReal string) string {
	d.mu.Lock()
	cmd, dest := d.cmd, d.destPath
	d.mu.Unlock()

	if cmd != nil && cmd.Process != nil && cmd.ProcessState == nil {
		out, err := exec.Command("lsof", "-p", strconv.Itoa(cmd.Process.Pid), "-Fn").Output()
		if err == nil {
			for _, l := range strings.Split(string(out), "\n") {
				if p, ok := strings.CutPrefix(l, "n"); ok &&
					strings.HasSuffix(p, ".part") && !strings.Contains(p, ".part-Frag") {
					if _, err := os.Stat(p); err == nil {
						return p
					}
				}
			}
		}
	}
	for _, c := range []string{dest + ".part", dest, pf.FinalPath + ".part", pf.FinalPath, finalReal} {
		if c != "" && c != ".part" {
			if _, err := os.Stat(c); err == nil {
				return c
			}
		}
	}
	// largest .part in the captured dest dir, else the final dir (largest, so
	// the assembled file wins over the little *-FragNN.part siblings)
	for _, dir := range []string{filepath.Dir(dest), pf.Dir} {
		if dir == "" || dir == "." {
			continue
		}
		matches, _ := filepath.Glob(filepath.Join(dir, "*.part"))
		var parts []string
		for _, m := range matches {
			if !strings.Contains(m, ".part-Frag") {
				parts = append(parts, m)
			}
		}
		sort.Slice(parts, func(i, j int) bool {
			si, _ := os.Stat(parts[i])
			sj, _ := os.Stat(parts[j])
			var a, b int64
			if si != nil {
				a = si.Size()
			}
			if sj != nil {
				b = sj.Size()
			}
			return a > b
		})
		if len(parts) > 0 {
			return parts[0]
		}
	}
	return ""
}

// OpenMPV streams the growing partial in mpv (detached).
func (d *Downloader) OpenMPV(path string) error {
	cmd := exec.Command("mpv",
		"--force-window=immediate", "--force-seekable=yes", "--cache=yes",
		"--demuxer-readahead-secs=20", "--profile=low-latency", "--", path)
	cmd.Stdout, cmd.Stderr = nil, nil
	if err := cmd.Start(); err != nil {
		return err
	}
	go func() { _ = cmd.Wait() }()
	return nil
}

func humanBytes(b int64) string {
	switch {
	case b >= 1<<30:
		return fmt.Sprintf("%.2fGiB", float64(b)/(1<<30))
	case b >= 1<<20:
		return fmt.Sprintf("%.1fMiB", float64(b)/(1<<20))
	case b >= 1<<10:
		return fmt.Sprintf("%.1fKiB", float64(b)/(1<<10))
	}
	return fmt.Sprintf("%dB", b)
}
