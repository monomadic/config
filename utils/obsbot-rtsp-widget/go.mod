module jayu/obsbot-rtsp-widget

go 1.23.6

// Share the free-disk-space-widget systray fork so the menu bar item resolves
// to this app (spatial AX) instead of Control Center. Stock v1.2.2 leaves it an
// unresolvable orphan, which makes Thaw spin up a phantom "Thaw Resolver"
// display on single-display/headless remote sessions.
replace github.com/getlantern/systray => ../free-disk-space-widget/third_party/systray

require github.com/getlantern/systray v1.2.2

require (
	github.com/getlantern/context v0.0.0-20190109183933-c447772a6520 // indirect
	github.com/getlantern/errors v0.0.0-20190325191628-abdb3e3e36f7 // indirect
	github.com/getlantern/golog v0.0.0-20190830074920-4ef2e798c2d7 // indirect
	github.com/getlantern/hex v0.0.0-20190417191902-c6586a6fe0b7 // indirect
	github.com/getlantern/hidden v0.0.0-20190325191715-f02dbb02be55 // indirect
	github.com/getlantern/ops v0.0.0-20190325191751-d70cb0d6f85f // indirect
	github.com/go-stack/stack v1.8.0 // indirect
	github.com/oxtoacart/bpool v0.0.0-20190530202638-03653db5a59c // indirect
	golang.org/x/sys v0.1.0 // indirect
)
