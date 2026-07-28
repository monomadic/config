package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
)

// Thumbnails share ytq's cache (YT_DLP_THUMBCACHE_DIR, default ~/.cache/ytq),
// named <extractor>_<id>.<ext> — a thumb either tool has fetched is reused.
// Rendering is chafa symbol art: plain ANSI text, so it composes with Bubble
// Tea's repaints (unlike kitty graphics, which fight them).

func thumbCacheDir() string {
	if d := os.Getenv("YT_DLP_THUMBCACHE_DIR"); d != "" {
		return d
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".cache", "ytq")
}

func cachedThumb(stem string) string {
	if stem == "" {
		return ""
	}
	dir := thumbCacheDir()
	for _, e := range []string{"jpg", "jpeg", "png", "webp"} {
		p := filepath.Join(dir, stem+"."+e)
		if _, err := os.Stat(p); err == nil {
			return p
		}
	}
	return ""
}

// fetchThumb ensures the thumbnail is in the cache (fetching via yt-dlp if
// needed), renders it with chafa, and sends it to the UI. Run in a goroutine.
func fetchThumb(p *tea.Program, pf *PreflightInfo, url string, cols, rows int) {
	if pf.ThumbStem == "" {
		return
	}
	file := cachedThumb(pf.ThumbStem)
	if file == "" {
		dir := thumbCacheDir()
		_ = os.MkdirAll(dir, 0o755)
		_ = exec.Command("yt-dlp",
			"--quiet", "--no-warnings", "--no-playlist", "--skip-download",
			"--no-download-archive", "--write-thumbnail", "--convert-thumbnails", "jpg",
			"-o", "thumbnail:"+filepath.Join(dir, "%(extractor)s_%(id)s.%(ext)s"),
			url).Run()
		file = cachedThumb(pf.ThumbStem)
	}
	if file == "" {
		return
	}
	if _, err := exec.LookPath("chafa"); err != nil {
		return
	}
	out, err := exec.Command("chafa",
		"--format", "symbols", "--animate", "off",
		"--size", fmt.Sprintf("%dx%d", cols, rows),
		file).Output()
	if err != nil || len(out) == 0 {
		return
	}
	p.Send(thumbMsg(strings.TrimRight(string(out), "\n")))
}
