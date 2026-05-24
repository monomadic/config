# System Uptime Widget

Simple macOS menu bar app that shows system uptime in a compact format.

## Build

```bash
go build -ldflags="-s -w" .
```

To update the repo-managed startup binary:

```bash
go build -ldflags="-s -w" -o ../../vendor/bin/system-uptime-widget .
```

## Run on login

From the dotfiles repo:

```bash
setup/install/system-uptime-widget.sh
```

This installs the repo-managed binary to `~/.local/bin/system-uptime-widget`,
writes `~/Library/LaunchAgents/com.jayu.system-uptime-widget.plist`, and
starts it with `launchctl`.

## Output

The menu bar title rounds to the nearest useful unit:

```text
30m
5h
2d
```
