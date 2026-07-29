package main

import (
	"bytes"
	"encoding/base64"
	"fmt"
	"image"
	_ "image/jpeg"
	"image/png"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
)

// Thumbnails share ytq's cache (YT_DLP_THUMBCACHE_DIR, default ~/.cache/ytq),
// named <extractor>_<id>.<ext> — a thumb either tool has fetched is reused.
//
// Rendering: on kitty-graphics terminals the image is transmitted once over
// /dev/tty and the view holds Unicode placeholder cells (U+10EEEE) — real text,
// so Bubble Tea repaints compose cleanly (classic kitty placements would not).
// Elsewhere, chafa symbol art as before.

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
// needed), renders it, and sends it to the UI. Run in a goroutine.
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
	if art := kittyThumb(file, cols, rows); art != "" {
		p.Send(thumbMsg(art))
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

// ---- kitty graphics (Unicode placeholder) rendering ----

// kittyImageID must stay < 256 so the placeholder can carry it in an 8-bit
// foreground colour (38;5;<id>).
const kittyImageID = 99

// Row/column diacritics from kitty's rowcolumn-diacritics table; index n marks
// row/column n. Only as many as the thumb box needs.
var kittyDiacritics = []rune{
	0x0305, 0x030D, 0x030E, 0x0310, 0x0312, 0x033D, 0x033E, 0x033F,
	0x0346, 0x034A, 0x034B, 0x034C, 0x0350, 0x0351, 0x0352, 0x0357,
}

func kittyCapable() bool {
	if os.Getenv("KITTY_WINDOW_ID") != "" {
		return true
	}
	term := os.Getenv("TERM")
	return strings.Contains(term, "kitty") || strings.Contains(term, "ghostty")
}

// kittyThumb transmits the image once (virtual placement) over /dev/tty and
// returns the placeholder-cell block for the view; "" means fall back to chafa.
func kittyThumb(file string, cols, rows int) string {
	if !kittyCapable() {
		return ""
	}
	f, err := os.Open(file)
	if err != nil {
		return ""
	}
	img, _, err := image.Decode(f)
	f.Close()
	if err != nil { // e.g. a webp in the cache — stdlib can't decode it
		return ""
	}

	// Fit into cols×rows preserving aspect; a terminal cell is ~1:2 (w:h).
	w, h := img.Bounds().Dx(), img.Bounds().Dy()
	if w < 1 || h < 1 {
		return ""
	}
	c, r := cols, (cols*h+2*w-1)/(2*w)
	if r > rows {
		r = rows
		c = (2 * rows * w) / h
	}
	if c < 1 {
		c = 1
	}
	if r < 1 {
		r = 1
	}
	if r > len(kittyDiacritics) {
		return ""
	}

	var pngBuf bytes.Buffer
	if err := png.Encode(&pngBuf, img); err != nil {
		return ""
	}

	tty, err := os.OpenFile("/dev/tty", os.O_WRONLY, 0)
	if err != nil {
		return ""
	}
	defer tty.Close()

	var seq bytes.Buffer
	// drop any previous transmission under this id, then transmit as a virtual
	// (U=1) placement, scaled to c×r cells; q=2 suppresses responses.
	fmt.Fprintf(&seq, "\x1b_Ga=d,d=I,i=%d,q=2\x1b\\", kittyImageID)
	data := base64.StdEncoding.EncodeToString(pngBuf.Bytes())
	first := true
	for len(data) > 0 {
		n := 4096
		if n > len(data) {
			n = len(data)
		}
		chunk, more := data[:n], len(data) > n
		m := 0
		if more {
			m = 1
		}
		if first {
			fmt.Fprintf(&seq, "\x1b_Ga=T,U=1,q=2,f=100,t=d,i=%d,c=%d,r=%d,m=%d;%s\x1b\\",
				kittyImageID, c, r, m, chunk)
			first = false
		} else {
			fmt.Fprintf(&seq, "\x1b_Gm=%d;%s\x1b\\", m, chunk)
		}
		data = data[n:]
	}
	if _, err := tty.Write(seq.Bytes()); err != nil {
		return ""
	}

	// Placeholder block: first cell of each line carries row+column diacritics,
	// the rest are bare and inferred (same row, column+1). The image id rides in
	// the foreground colour.
	var b strings.Builder
	for row := 0; row < r; row++ {
		fmt.Fprintf(&b, "\x1b[38;5;%dm", kittyImageID)
		b.WriteRune('\U0010EEEE')
		b.WriteRune(kittyDiacritics[row])
		b.WriteRune(kittyDiacritics[0])
		for col := 1; col < c; col++ {
			b.WriteRune('\U0010EEEE')
		}
		b.WriteString("\x1b[39m")
		if row < r-1 {
			b.WriteByte('\n')
		}
	}
	return b.String()
}
