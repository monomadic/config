package main

import (
	"context"
	"encoding/json"
	"errors"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
)

// Media inspection helpers. These mirror the checks `media-audit` performs so
// that a spill run and an audit run agree on what counts as a problem file:
// the same faststart atom walk, the same leading-black heuristic, the same
// source-URL tag list, and the same "★★★☆☆ in the filename" rating convention
// written by `media-set-rating`.

var errNoFFprobe = errors.New("ffprobe not found")

// faststart states, matching media-audit's exit codes: moov before mdat is
// good, mdat first means the file needs a faststart rewrite, and anything we
// can't parse is inconclusive.
const (
	faststartOK = iota
	faststartMissing
	faststartUnknown
)

func isFaststartPath(path string) bool {
	switch strings.ToLower(filepath.Ext(path)) {
	case ".mp4", ".mov", ".m4v":
		return true
	}
	return false
}

func isVideoPath(path string) bool {
	return videoExts[strings.ToLower(filepath.Ext(path))]
}

// ratingFromName reads the trailing " ★★★☆☆" rating out of a filename stem,
// returning 0 when the file carries no rating.
func ratingFromName(path string) int {
	stem := strings.TrimSuffix(filepath.Base(path), filepath.Ext(path))
	r := []rune(stem)
	if len(r) < 6 || r[len(r)-6] != ' ' {
		return 0
	}
	n := 0
	for _, c := range r[len(r)-5:] {
		switch c {
		case '★':
			n++
		case '☆':
		default:
			return 0
		}
	}
	return n
}

type ffStream struct {
	Width        int    `json:"width"`
	Height       int    `json:"height"`
	AvgFrameRate string `json:"avg_frame_rate"`
	RFrameRate   string `json:"r_frame_rate"`
}

// probeVideo returns the short side (so portrait 1080x1920 still reads as
// 1080p) and the frame rate of a file's first video stream.
func probeVideo(ctx context.Context, path string) (side int, fps float64, err error) {
	var out struct {
		Streams []ffStream `json:"streams"`
	}
	err = ffprobeJSON(ctx, &out, "-v", "error", "-select_streams", "v:0",
		"-show_entries", "stream=width,height,avg_frame_rate,r_frame_rate",
		"-of", "json", "--", path)
	if err != nil {
		return 0, 0, err
	}
	if len(out.Streams) == 0 || out.Streams[0].Width == 0 || out.Streams[0].Height == 0 {
		return 0, 0, errors.New("no video stream")
	}
	s := out.Streams[0]
	side = s.Height
	if s.Width < side {
		side = s.Width
	}
	fps = parseRate(s.AvgFrameRate)
	if fps <= 0 {
		fps = parseRate(s.RFrameRate)
	}
	return side, fps, nil
}

// parseRate turns an ffprobe rational like "60000/1001" into 59.94.
func parseRate(s string) float64 {
	num, den, ok := strings.Cut(s, "/")
	n, err := strconv.ParseFloat(strings.TrimSpace(num), 64)
	if err != nil {
		return 0
	}
	if !ok {
		return n
	}
	d, err := strconv.ParseFloat(strings.TrimSpace(den), 64)
	if err != nil || d == 0 {
		return 0
	}
	return n / d
}

var urlRe = regexp.MustCompile(`https?://[^\s"<>]+`)

// urlTagKeys is media-audit's search order for an embedded source URL.
var urlTagKeys = []string{"source_url", "webpage_url", "purl", "url", "comment", "description"}

// probeSourceURL returns the embedded source URL of a media file, or "" when
// no tag carries one. A tag literally set to "none" counts as no URL.
func probeSourceURL(ctx context.Context, path string) (string, error) {
	var out struct {
		Format struct {
			Tags map[string]string `json:"tags"`
		} `json:"format"`
	}
	if err := ffprobeJSON(ctx, &out, "-v", "error", "-show_entries", "format_tags",
		"-of", "json", "--", path); err != nil {
		return "", err
	}
	tags := make(map[string]string, len(out.Format.Tags))
	for k, v := range out.Format.Tags {
		tags[strings.ToLower(k)] = v
	}
	for _, key := range urlTagKeys {
		v := strings.TrimSpace(tags[key])
		if v == "" || strings.EqualFold(v, "none") {
			continue
		}
		if u := urlRe.FindString(v); u != "" {
			return u, nil
		}
	}
	return "", nil
}

func ffprobeJSON(ctx context.Context, v any, args ...string) error {
	if !haveCmd("ffprobe") {
		return errNoFFprobe
	}
	out, err := exec.CommandContext(ctx, "ffprobe", args...).Output()
	if err != nil {
		// ffprobe's stderr is a wall of detail; for a file list, "it isn't
		// media" is the only distinction worth showing.
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			return errNotMedia
		}
		return err
	}
	return json.Unmarshal(out, v)
}

var errNotMedia = errors.New("not a media file")

// faststartState walks the top-level MP4/MOV atoms and reports whether `moov`
// precedes `mdat` — the same check media-audit runs before offering a
// faststart rewrite.
func faststartState(path string) int {
	f, err := os.Open(path)
	if err != nil {
		return faststartUnknown
	}
	defer f.Close()

	info, err := f.Stat()
	if err != nil {
		return faststartUnknown
	}
	size := info.Size()

	header := make([]byte, 16)
	var pos int64
	for i := 0; i < 100000; i++ {
		if pos+8 > size {
			return faststartUnknown
		}
		if _, err := f.ReadAt(header[:8], pos); err != nil {
			return faststartUnknown
		}
		atomSize := int64(be32(header[0:4]))
		atomType := string(header[4:8])
		headerLen := int64(8)
		switch atomSize {
		case 1:
			if _, err := f.ReadAt(header[8:16], pos+8); err != nil {
				return faststartUnknown
			}
			atomSize = int64(be64(header[8:16]))
			headerLen = 16
		case 0:
			atomSize = size - pos
		}
		if atomSize < headerLen {
			return faststartUnknown
		}
		switch atomType {
		case "moov":
			return faststartOK
		case "mdat":
			return faststartMissing
		}
		pos += atomSize
	}
	return faststartUnknown
}

func be32(b []byte) uint32 {
	return uint32(b[0])<<24 | uint32(b[1])<<16 | uint32(b[2])<<8 | uint32(b[3])
}

func be64(b []byte) uint64 {
	return uint64(be32(b[0:4]))<<32 | uint64(be32(b[4:8]))
}

var blackRe = regexp.MustCompile(`black_start:([0-9.]+)\s+black_end:([0-9.]+)`)

// Same knobs as ffmpeg-detect-black-scenes: scan the first 10s for black runs
// of at least 0.2s.
const (
	blackScanSeconds = "10"
	blackDetectVF    = "blackdetect=d=0.2:pix_th=0.08:pic_th=0.98"
)

// blackIntroSeconds returns the length of a black intro at the head of a
// video, or 0 when there is none. media-audit's rule: the run has to start
// within the first 0.25s and last past 0.05s.
func blackIntroSeconds(ctx context.Context, path string) (float64, error) {
	if !haveCmd("ffmpeg") {
		return 0, errors.New("ffmpeg not found")
	}
	cmd := exec.CommandContext(ctx, "ffmpeg", "-hide_banner", "-nostats",
		"-t", blackScanSeconds, "-i", path, "-vf", blackDetectVF, "-an", "-f", "null", "-")
	// blackdetect reports on stderr; a non-zero exit still leaves usable lines.
	out, err := cmd.CombinedOutput()
	if err != nil && len(out) == 0 {
		return 0, err
	}
	if ctx.Err() != nil {
		return 0, ctx.Err()
	}
	for _, m := range blackRe.FindAllStringSubmatch(string(out), -1) {
		start, err1 := strconv.ParseFloat(m[1], 64)
		end, err2 := strconv.ParseFloat(m[2], 64)
		if err1 != nil || err2 != nil {
			continue
		}
		if start <= 0.25 && end > 0.05 {
			return end, nil
		}
	}
	return 0, nil
}
