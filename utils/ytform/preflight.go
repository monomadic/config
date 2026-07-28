package main

import (
	"fmt"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
)

// PreflightInfo is what one --simulate extraction tells us up front: where the
// file will land and the metadata that seeds the form.
type PreflightInfo struct {
	FinalPath string // yt-dlp's own proposed output path
	Dir       string
	Ext       string
	ThumbStem string // <extractor>_<id> — ytq's shared thumb-cache stem
	Gallery   bool   // the URL itself is a playlist/channel/model page
	Meta      Meta
}

// Preflight runs one --simulate extraction. --print honors the yt-dlp config
// (aliases, paths), so the filename matches what the real run will write.
//
// --no-download-archive matters: an already-archived id is skipped BEFORE
// extraction, printing nothing — this is an explicit single-URL tool, so the
// archive is overridden on every yt-dlp call.
//
// -I 1:1 keeps this to ONE extraction even for an index page; the playlist
// prints are how we detect that case (and bail — ytui handles galleries).
func Preflight(passthru []string, url string) (*PreflightInfo, error) {
	args := []string{
		"--no-warnings", "--no-playlist", "--simulate", "--no-download-archive",
		"-I", "1:1",
		"--print", "filename",
		"--print", "%(title)s",
		"--print", "%(channel,uploader)s",
		"--print", "%(cast)l",
		"--print", "%(extractor)s_%(id)s",
		"--print", "%(playlist_count)s",
		"--print", "%(playlist,playlist_id)s",
	}
	args = append(args, passthru...)
	args = append(args, url)

	out, err := exec.Command("yt-dlp", args...).Output()
	if err != nil {
		msg := ""
		if ee, ok := err.(*exec.ExitError); ok {
			msg = strings.TrimSpace(string(ee.Stderr))
		}
		return nil, fmt.Errorf("could not resolve the video: %v\n%s", err, msg)
	}
	lines := strings.Split(strings.TrimRight(string(out), "\n"), "\n")
	if len(lines) < 7 || lines[0] == "" {
		return nil, fmt.Errorf("could not resolve the video (unexpected yt-dlp output)")
	}

	pf := &PreflightInfo{FinalPath: lines[0]}
	if i := strings.LastIndexByte(pf.FinalPath, '/'); i >= 0 {
		pf.Dir = pf.FinalPath[:i]
	}
	if i := strings.LastIndexByte(pf.FinalPath, '.'); i > strings.LastIndexByte(pf.FinalPath, '/') {
		pf.Ext = pf.FinalPath[i+1:]
	}
	pf.Meta.Title = strings.TrimSpace(na(lines[1]))
	pf.Meta.Channel = strings.TrimSpace(na(lines[2]))
	pf.Meta.Actors = SplitList(na(lines[3]))
	pf.ThumbStem = Sanitize(lines[4])

	// A missing field prints as the literal "NA" unless the config sets
	// --output-na-placeholder "" — both spellings of "absent" count here.
	count, _ := strconv.Atoi(regexp.MustCompile(`[^0-9]`).ReplaceAllString(lines[5], ""))
	plname := strings.TrimSpace(na(lines[6]))
	if plname != "" || count > 1 {
		pf.Gallery = true
	}
	return pf, nil
}

// na maps yt-dlp's "missing field" placeholders to the empty string.
func na(s string) string {
	switch strings.TrimSpace(s) {
	case "NA", "none":
		return ""
	}
	return s
}
