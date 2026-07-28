package main

import "strings"

// Meta is the proposed-filename metadata, editable in the form while the
// download runs. Stem() is the reverse of media-parse-filename-to-json:
//
//	Actor A, Actor B - [Channel] Title #tag1 #tag2 (Origin) ★★★☆☆
type Meta struct {
	Title   string
	Channel string
	Origin  string
	Actors  []string
	Tags    []string
	Rating  int
}

func (m Meta) Equal(o Meta) bool {
	return m.Title == o.Title && m.Channel == o.Channel && m.Origin == o.Origin &&
		m.Rating == o.Rating &&
		strings.Join(m.Actors, "\x00") == strings.Join(o.Actors, "\x00") &&
		strings.Join(m.Tags, "\x00") == strings.Join(o.Tags, "\x00")
}

// Stem composes the filename (no extension) from the fields. A channel that
// just restates the first actor's name adds nothing and is dropped.
func (m Meta) Stem() string {
	ch := m.Channel
	if len(m.Actors) > 0 && strings.EqualFold(ch, m.Actors[0]) {
		ch = ""
	}
	var b strings.Builder
	if len(m.Actors) > 0 {
		b.WriteString(strings.Join(m.Actors, ", "))
		b.WriteString(" - ")
	}
	if ch != "" {
		b.WriteString("[" + ch + "] ")
	}
	b.WriteString(m.Title)
	for _, t := range m.Tags {
		b.WriteString(" #" + t)
	}
	if m.Origin != "" {
		b.WriteString(" (" + m.Origin + ")")
	}
	if m.Rating > 0 {
		b.WriteString(" " + StarsPlain(m.Rating))
	}
	return Sanitize(b.String())
}

// StarsPlain is the form stored in the filename: exactly five ★/☆, no spaces
// (same grammar as media-set-rating).
func StarsPlain(n int) string {
	var b strings.Builder
	for i := 1; i <= 5; i++ {
		if i <= n {
			b.WriteRune('★')
		} else {
			b.WriteRune('☆')
		}
	}
	return b.String()
}

// Sanitize makes a filename component filesystem-safe: no slashes or
// control whitespace, trimmed.
func Sanitize(s string) string {
	s = strings.ReplaceAll(s, "/", "-")
	s = strings.ReplaceAll(s, "\n", " ")
	s = strings.ReplaceAll(s, "\t", " ")
	return strings.TrimSpace(s)
}

// SplitList parses a comma-separated form field into trimmed, non-empty items.
func SplitList(s string) []string {
	var out []string
	for _, p := range strings.Split(s, ",") {
		if p = strings.TrimSpace(p); p != "" {
			out = append(out, p)
		}
	}
	return out
}

// SplitTags parses the tags field: comma- or space-separated, leading '#'
// tolerated so pasting "#a #b" round-trips.
func SplitTags(s string) []string {
	var out []string
	for _, p := range strings.FieldsFunc(s, func(r rune) bool { return r == ',' || r == ' ' }) {
		p = strings.TrimSpace(strings.TrimPrefix(p, "#"))
		if p != "" {
			out = append(out, p)
		}
	}
	return out
}
