package main

import "testing"

func TestParseLaunchState(t *testing.T) {
	output := `gui/501/local.obsbot-rtsp = {
	active count = 1
	path = /Users/nom/Library/LaunchAgents/local.obsbot-rtsp.plist
	state = running
}`

	if got := parseLaunchState(output); got != "running" {
		t.Fatalf("parseLaunchState() = %q, want running", got)
	}
}

func TestTitleForStatuses(t *testing.T) {
	running := serviceStatus{Loaded: true, Running: true, State: "running"}
	loaded := serviceStatus{Loaded: true, State: "exited"}
	stopped := serviceStatus{}

	if got := titleForStatuses(running, running); got != runningIcon+" RTSP" {
		t.Fatalf("all running title = %q", got)
	}
	if got := titleForStatuses(running, stopped); got != warningIcon+" RTSP" {
		t.Fatalf("partial running title = %q", got)
	}
	if got := titleForStatuses(stopped, loaded); got != warningIcon+" RTSP" {
		t.Fatalf("loaded non-running title = %q", got)
	}
	if got := titleForStatuses(stopped, stopped); got != stoppedIcon+" RTSP" {
		t.Fatalf("stopped title = %q", got)
	}
}
