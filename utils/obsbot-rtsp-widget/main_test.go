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

func TestTitleForStatus(t *testing.T) {
	if got := titleForStatus(serviceStatus{Loaded: true, Running: true}); got != runningIcon+" RTSP" {
		t.Fatalf("running title = %q", got)
	}
	if got := titleForStatus(serviceStatus{Loaded: true, State: "exited"}); got != warningIcon+" RTSP" {
		t.Fatalf("loaded non-running title = %q", got)
	}
	if got := titleForStatus(serviceStatus{}); got != stoppedIcon+" RTSP" {
		t.Fatalf("stopped title = %q", got)
	}
}
