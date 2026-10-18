# cpu-usage-widget

macOS menu bar CPU (and optionally GPU) load meter, in the same family as
`battery-widget` and `free-disk-space-widget`. Rust, talking to AppKit directly
via [objc2](https://github.com/madsmtm/objc2) — no wrapper library, no vendored
forks, no `.app` bundle. The dropdown is a real `NSMenu` assigned to the status
item, so macOS presents it natively and rebuilds it from the latest sample each
time it opens.

## Where the numbers come from

**CPU** — one `host_processor_info` / `PROCESSOR_CPU_LOAD_INFO` call every two
seconds, differenced against the previous sample. No `top`, no `ps`, so the
meter itself costs microseconds a tick. Utilization is a rate rather than a
reading, so the first tick after launch reports zero: that is the baseline being
primed.

**GPU** — `Device Utilization %` from the `PerformanceStatistics` dictionary on
the `IOAccelerator` IOKit service, which is the same number Activity Monitor's
GPU History graph plots. It needs no elevated privileges, unlike `powermetrics`.
On a machine with more than one accelerator the busiest one wins. Already a
rate, so a single read is a complete answer.

**Top processes** — the dropdown names the three of your processes that used
the most CPU over the last interval. Each tick reads every process's cumulative
user+system time with `proc_pidinfo` and differences it against the previous
tick, the way `top` does; that is a millisecond or two for a few hundred
processes, and only the winners have their names and icons resolved. Without
root only your own processes are readable — `top` and `ps` are setuid root,
which is how they see `kernel_task`, `WindowServer` and `coreaudiod` — so those
never appear here. Rolling what they use into a row of its own was tried and
dropped: it topped the list nearly always and said nothing you could act on.

## The dropdown

The top of the menu is a dashboard drawn by two custom views (`dashboard.rs`),
designed in Figma first: the Mac's name, a **CPU** badge over its load as a big
number, one pill per core (red when a core is at 80% or more), and, when the
GPU is shown, a **GPU** badge and number on the right over three pills for
device, renderer and tiler. Below a rule, **Top processes** lists each process
with its app icon (from the `.app` bundle around its executable, via
`NSWorkspace`), a meter and its share in Activity Monitor's convention — 100%
is one core, the meter stops there, and the number keeps counting; a process
holding a whole core or more gets the hot colour. Under that, **Style** and
**Show** are native submenus whose rows also show the current choice, then
**Open Activity Monitor** and **Quit**.

Nothing in the dashboard is a fixed point size: every dimension is a ratio of
the menu font's size, and every colour is a system semantic or accent colour,
so it follows the system text size and light/dark appearance.

## Layout styles

Pick from the Style submenu.

| Style | Menu bar |
|---|---|
| Per-core Bars (default) | one vertical bar per core, height ∝ that core's load |
| Aggregate Bar | a single fill bar for overall utilization |
| Aggregate Bar and Text | the bar plus a percentage |
| Percentage Text | overall utilization as text only |

## Showing the GPU

The Show submenu picks which meters share the item: **CPU**, **GPU**, or **CPU
and GPU**. With both, every style stacks two rows in one image — CPU always on
top — and the two text layouts pick up a leading `C` / `G` so it is clear which
row is which.

Per-core Bars is the exception. There is no per-GPU-core load to draw: the
driver publishes the core count as static configuration (`num_cores` in
`GPUConfigurationVariable`), but every utilization figure it reports is
whole-device, and `powermetrics` has no per-core breakdown either. So the GPU
contributes its three figures as three columns after a wider gap — device,
then renderer and tiler, which is what tells a shading-bound load from a
geometry-bound one. Those two also appear in the dropdown.

If the machine reports no accelerator, the GPU meter simply drops out and the
CPU keeps the item to itself; the dropdown says so.

## Sizing

There are no hardcoded point sizes. The font is whatever macOS reports as the
menu bar font, the canvas is the status bar's own thickness, and every bar, gap
and pad is a ratio of the font's metrics — change the system text size and the
widget follows. Everything drawn is a template image, so macOS tints it to
match the menu bar in light and dark appearance.

## Settings

Selections are saved as `key=value` lines to:

```text
~/.config/cpu-usage-widget/settings
```

The Go implementation kept a `settings.json` under
`~/Library/Application Support/cpu-usage-widget/`; that file is no longer read.

## Build

```bash
cargo build --release
```

To update the repo-managed startup binary:

```bash
cargo build --release && cp target/release/cpu-usage-widget ../../vendor/bin/cpu-usage-widget
```

## Run on login

From the dotfiles repo:

```bash
scripts/install/install-cpu-usage-widget.sh
```

This installs the repo-managed binary to `~/.local/bin/cpu-usage-widget`, writes
`~/Library/LaunchAgents/com.jayu.cpu-usage-widget.plist`, and starts it with
`launchctl`.
