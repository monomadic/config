package main

import "testing"

func TestStemGrammar(t *testing.T) {
	cases := []struct {
		name string
		m    Meta
		want string
	}{
		{"full", Meta{
			Title: "Some Title", Channel: "Chan", Origin: "Site",
			Actors: []string{"Actor A", "Actor B"}, Tags: []string{"t1", "t2"}, Rating: 3,
		}, "Actor A, Actor B - [Chan] Some Title #t1 #t2 (Site) ★★★☆☆"},
		{"title only", Meta{Title: "Just This"}, "Just This"},
		{"channel equals first actor (case-insensitive) is dropped", Meta{
			Title: "T", Channel: "jane doe", Actors: []string{"Jane Doe"},
		}, "Jane Doe - T"},
		{"channel kept when it differs from first actor", Meta{
			Title: "T", Channel: "Studio", Actors: []string{"Jane Doe"},
		}, "Jane Doe - [Studio] T"},
		{"slashes sanitized", Meta{Title: "a/b"}, "a-b"},
		{"zero rating omitted", Meta{Title: "T", Rating: 0}, "T"},
		{"five stars", Meta{Title: "T", Rating: 5}, "T ★★★★★"},
	}
	for _, c := range cases {
		if got := c.m.Stem(); got != c.want {
			t.Errorf("%s: got %q, want %q", c.name, got, c.want)
		}
	}
}

func TestSplitters(t *testing.T) {
	if got := SplitList(" a , b,, c "); len(got) != 3 || got[0] != "a" || got[2] != "c" {
		t.Errorf("SplitList: %v", got)
	}
	if got := SplitTags("#a b, #c"); len(got) != 3 || got[0] != "a" || got[2] != "c" {
		t.Errorf("SplitTags: %v", got)
	}
}

func TestMetaEqual(t *testing.T) {
	a := Meta{Title: "T", Actors: []string{"x"}}
	b := a
	if !a.Equal(b) {
		t.Error("identical metas not equal")
	}
	b.Rating = 4
	if a.Equal(b) {
		t.Error("rating change not detected")
	}
}
