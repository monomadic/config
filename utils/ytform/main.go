// ytform — yt-dlp downloader with a live metadata form.
//
// The Go sibling of config/zsh/bin/ytui: same preflight, same progress
// template, same filename grammar — but the metadata (title, actors, channel,
// origin, tags, rating) is a real form you fill out WHILE the file downloads,
// instead of ytui's modal-at-a-time keys.
//
//	ytform [yt-dlp args...] <url>     # URL last, rest passes through to yt-dlp
//
// Keys: tab/↑↓ move between fields · type to edit · rating row takes 0-5/←→
// ctrl+o stream the growing .part in mpv · ctrl+p pause/resume · esc cancel
// When the download finishes, enter applies the form: if you changed anything,
// the file is renamed to  Actor A, Actor B - [Channel] Title #tag (Origin) ★★★☆☆.ext
// (the same grammar media-parse-filename-to-json reads back). Untouched form →
// yt-dlp's own filename is left alone.
//
// Galleries (playlists, model pages) are out of scope — use ytui for those.
// Optional runtime deps: mpv (ctrl+o), chafa (thumbnail), lsof (.part lookup).
package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
)

func usage() {
	fmt.Fprint(os.Stderr, `Usage: ytform [yt-dlp args...] <url>

Download <url> with yt-dlp behind a TUI with a live metadata form. Everything
before the URL is forwarded to yt-dlp, so config aliases work.

Keys:  tab/up/down fields · 0-5 rating · ctrl+o mpv · ctrl+p pause · esc cancel
       enter (when finished) apply form & exit
`)
}

func die(format string, a ...any) {
	fmt.Fprintf(os.Stderr, "ytform: "+format+"\n", a...)
	os.Exit(1)
}

func main() {
	args := os.Args[1:]
	if len(args) == 0 || args[0] == "-h" || args[0] == "--help" {
		usage()
		if len(args) == 0 {
			os.Exit(2)
		}
		return
	}
	url := args[len(args)-1]
	passthru := args[:len(args)-1]
	if url == "" || strings.HasPrefix(url, "-") {
		die("last argument must be a URL (got: %s)", url)
	}
	if _, err := exec.LookPath("yt-dlp"); err != nil {
		die("yt-dlp not found")
	}

	pf, err := Preflight(passthru, url)
	if err != nil {
		die("%v", err)
	}
	if pf.Gallery {
		die("this URL is a gallery/playlist — use ytui for those")
	}
	if pf.Dir != "" {
		_ = os.MkdirAll(pf.Dir, 0o755)
	}

	dl := &Downloader{}
	p := tea.NewProgram(newModel(pf, dl, url), tea.WithAltScreen())
	go dl.Run(p, passthru, url)
	go fetchThumb(p, pf, url, 32, 9)

	final, err := p.Run()
	if err != nil {
		die("%v", err)
	}
	m := final.(model)

	if m.cancelled {
		fmt.Fprintln(os.Stderr, "ytform: cancelled")
		os.Exit(130)
	}
	if m.exitCode != 0 {
		fmt.Fprintf(os.Stderr, "ytform: yt-dlp failed (exit %d)\n", m.exitCode)
		for _, l := range dl.LogTail(14) {
			fmt.Fprintln(os.Stderr, "  "+l)
		}
		os.Exit(m.exitCode)
	}

	src := m.finalReal
	if src == "" {
		src = pf.FinalPath
	}
	if _, err := os.Stat(src); err != nil {
		fmt.Printf("ytform: finished, but couldn't locate the output file\n  expected: %s\n", src)
		return
	}
	fmt.Printf("✓ downloaded  %s\n", filepath.Base(src))

	// Rename only if the form was actually touched — an untouched run keeps
	// yt-dlp's own filename, same contract as ytui.
	meta := m.meta()
	if meta.Equal(m.initial) {
		return
	}
	ext := strings.TrimPrefix(filepath.Ext(src), ".")
	dst := filepath.Join(filepath.Dir(src), meta.Stem()+"."+ext)
	switch {
	case dst == src:
		fmt.Println("  name unchanged")
	case exists(dst):
		fmt.Printf("  rename skipped — exists: %s\n", filepath.Base(dst))
	default:
		if err := os.Rename(src, dst); err != nil {
			fmt.Fprintf(os.Stderr, "  rename failed: %v\n", err)
			os.Exit(1)
		}
		fmt.Printf("  renamed → %s\n", filepath.Base(dst))
	}
}

func exists(p string) bool {
	_, err := os.Stat(p)
	return err == nil
}
