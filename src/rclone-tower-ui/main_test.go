package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func TestTelemetryAndWarning(t *testing.T) {
	m := model{}
	m.consume(`{"level":"info","stats":{"listed":42,"checks":12,"bytes":100,"totalBytes":200,"transferring":[{"name":"movie.mp4","percentage":50}]}}`)
	if m.stats.Listed != 42 || len(m.stats.Transferring) != 1 {
		t.Fatal("lost scan/transfer telemetry")
	}
	m.consume(`{"level":"notice","object":"link","msg":"Can't follow symlink without -L/--copy-links"}`)
	if m.warnings != 1 || !strings.Contains(m.lastWarning, "symlink") {
		t.Fatal("skipped file warning hidden")
	}
}

func TestLogRotationAndPartialRecord(t *testing.T) {
	p := filepath.Join(t.TempDir(), "sync.log")
	os.WriteFile(p, []byte("first\npart"), 0600)
	r := logReader{path: p}
	if got := r.read(); len(got) != 1 || got[0] != "first" {
		t.Fatal(got)
	}
	os.Rename(p, p+".old")
	os.WriteFile(p, []byte("replacement record longer than old log\n"), 0600)
	if got := r.read(); len(got) != 1 || got[0] != "replacement record longer than old log" {
		t.Fatal(got)
	}
}

func TestStopWaitsAndPreservesFailure(t *testing.T) {
	m := model{started: time.Now(), reader: &logReader{}}
	u, _ := m.Update(doneMsg(7))
	got := u.(model)
	if !got.done || got.code != 7 || strings.Contains(got.phase, "complete") {
		t.Fatal(got.phase)
	}
}

func TestUntrustedTerminalText(t *testing.T) {
	if got := clean("clip\x1b[2J\nname"); got != "clip name" {
		t.Fatal(got)
	}
}
