package main

import (
	"context"
	"crypto/md5"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

// Fixed thumbnail cell box. Chafa renders into this regardless of terminal
// size; the UI reserves a matching column for it beside the progress readout.
const (
	thumbCols = 26
	thumbRows = 13
)

var (
	imageExts = map[string]bool{
		".jpg": true, ".jpeg": true, ".png": true, ".gif": true, ".webp": true,
		".bmp": true, ".tif": true, ".tiff": true, ".heic": true, ".heif": true, ".avif": true,
	}
	videoExts = map[string]bool{
		".mp4": true, ".mov": true, ".m4v": true, ".mkv": true, ".avi": true,
		".webm": true, ".wmv": true, ".flv": true, ".m2ts": true, ".ts": true, ".mpg": true, ".mpeg": true,
	}
)

func haveCmd(name string) bool {
	_, err := exec.LookPath(name)
	return err == nil
}

// renderThumb returns a chafa "symbols" rendering (plain ANSI text, so it
// composes with the Bubble Tea framebuffer) of a preview for path, or "" when
// no preview is possible. Everything here is best-effort and time-boxed so it
// never stalls the copy loop.
func renderThumb(path string, cols, rows int) string {
	if !haveCmd("chafa") {
		return ""
	}
	ext := strings.ToLower(filepath.Ext(path))

	var img string
	switch {
	case imageExts[ext]:
		img = path
	case videoExts[ext]:
		img = videoFrame(path)
	default:
		return ""
	}
	if img == "" {
		return ""
	}

	ctx, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()
	cmd := exec.CommandContext(ctx, "chafa",
		"-f", "symbols",
		"--animate", "off",
		"--polite", "on",
		"--size", fmt.Sprintf("%dx%d", cols, rows),
		img,
	)
	out, err := cmd.Output()
	if err != nil {
		return ""
	}
	return strings.TrimRight(string(out), "\n")
}

// videoFrame extracts a representative frame with ffmpeg, cached by path +
// mtime + size so revisits are instant. Returns the cached jpg path or "".
func videoFrame(path string) string {
	if !haveCmd("ffmpeg") {
		return ""
	}
	info, err := os.Stat(path)
	if err != nil {
		return ""
	}
	sum := md5.Sum([]byte(fmt.Sprintf("%s:%d:%d", path, info.ModTime().UnixNano(), info.Size())))
	cacheDir := filepath.Join(cacheHome(), "spill", "thumbs")
	if err := os.MkdirAll(cacheDir, 0o755); err != nil {
		return ""
	}
	out := filepath.Join(cacheDir, fmt.Sprintf("%x.jpg", sum))
	if fi, err := os.Stat(out); err == nil && fi.Size() > 0 {
		return out
	}

	// A couple of seconds in usually clears any black leader; fall back to 0.
	for _, ss := range []string{"2", "0"} {
		ctx, cancel := context.WithTimeout(context.Background(), 8*time.Second)
		cmd := exec.CommandContext(ctx, "ffmpeg", "-v", "error", "-y",
			"-ss", ss, "-i", path, "-frames:v", "1",
			"-vf", "scale=480:-2:flags=lanczos", "-q:v", "3", out)
		err := cmd.Run()
		cancel()
		if err == nil {
			if fi, serr := os.Stat(out); serr == nil && fi.Size() > 0 {
				return out
			}
		}
	}
	_ = os.Remove(out)
	return ""
}

func cacheHome() string {
	if x := os.Getenv("XDG_CACHE_HOME"); x != "" {
		return x
	}
	if home, err := os.UserHomeDir(); err == nil {
		return filepath.Join(home, ".cache")
	}
	return os.TempDir()
}
