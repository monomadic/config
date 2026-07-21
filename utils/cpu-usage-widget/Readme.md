# CPU Usage Widget

Simple macOS menu bar app that shows per-core CPU utilization.

Companion to [free-disk-space-widget](../free-disk-space-widget/). It reads the
kernel's cumulative CPU tick counters via a single Mach call
(`host_processor_info` / `PROCESSOR_CPU_LOAD_INFO`) every 2 seconds and diffs
successive samples — no `top`/`ps` shelling, no process enumeration, so it costs
microseconds per tick.

## Menu options

Layout styles:

- **Per-core Bars** (default) — one vertical bar per core, height ∝ that core's load.
- **Aggregate Bar** — a single fill bar for overall utilization.
- **Aggregate Bar and Text** — the bar plus a percentage.
- **Percentage Text** — overall utilization as text only.

Selections are saved to:

```text
~/Library/Application Support/cpu-usage-widget/settings.json
```

## Build

```bash
go build -ldflags="-s -w" .
```

To update the repo-managed startup binary:

```bash
go build -ldflags="-s -w" -o ../../vendor/bin/cpu-usage-widget .
```

## Run on login

From the dotfiles repo:

```bash
setup/install/cpu-usage-widget.sh
```

This installs the repo-managed binary to `~/.local/bin/cpu-usage-widget`, writes
`~/Library/LaunchAgents/com.jayu.cpu-usage-widget.plist`, and starts it with
`launchctl`.
