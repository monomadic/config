# AGENTS.md

Personal dotfiles repo, deployed with plain zsh via `Deployfile`.
**macOS only** — Linux config (i3, sway, waybar, foot, weston, refind, ...) has been removed; don't add it back.
Full layout rules: [docs/STRUCTURE.md](docs/STRUCTURE.md). Bootstrap docs: [README.md](README.md).

`CLAUDE.md` is a symlink to this file — edit `AGENTS.md`, never the symlink.

## The one thing to understand first

The linker **symlinks** repo files into place.
`config/yazi/` IS `~/.config/yazi/` on this machine. Editing a file in this repo
changes the live config immediately — no build, no deploy. Treat edits to `config/`
as live changes, and don't "install" anything by copying files out of the repo.

A deploy run is only needed when the *mapping* changes: a new package, a new
call in `Deployfile`, or a changed target path.

## The repo is location-independent

Nothing assumes `~/config`. `scripts/**` scripts resolve the root from their own
path (`dirname $0/../..`), `bin/deploy` resolves through its
symlink with `${0:A}` (`:h:h` — it sits one level down), and `.zshenv` derives
`$DOTFILES_DIR` the same way.
When you write a new script, follow that pattern — never hardcode `$HOME/config`,
and prefer `$DOTFILES_DIR` in shell config.

Tool configs that need to call a repo script should reference the **deployed**
path (`~/.local/bin/foo`, `~/.config/kitty/…`), not the repo path — the symlink is
stable wherever the checkout lives. Known exception: `config/zellij/layouts/*.kdl`
still holds absolute paths because Zellij's KDL does no env/tilde expansion.

Those absolute `~/.local/bin/...` paths are deliberate, not laziness: yazi,
mpv, karabiner, kitty, motherfucker and switchblade are launched by the GUI,
which inherits launchd's bare `PATH`, not your shell's. A bare command name
works from a shell script but silently fails from those configs. Inside a
script that already has a login shell's `PATH`, prefer the bare name.

## How deployment works

`Deployfile` is plain zsh sourced by `scripts/setup/link.zsh`:

```zsh
link_file config/zsh/zshrc.zsh "$HOME/.zshrc"
link_tree bin "$HOME/.local/bin"
```

`link_file` links a file or whole directory; `link_tree` links files recursively
(trailing arguments are ignore patterns, relative to its source); `ensure_dir`
creates real directories for data that must exist but never be linked.
Add or comment out calls directly. There is no package/profile system, parser,
state database, host override, or pruning. Existing conflicts are preserved and
reported, while the remaining mappings continue. The one exception is a
dangling symlink at the target or a directory above it: it holds nothing, so
it is removed (and reported) and the link goes in. Final status is nonzero on
failure. Do not restore the old deployment system.

- `scripts/setup/bootstrap.sh`: download archive if needed, then link; built-in macOS tools only.
- `scripts/setup/deploy.sh` or `deploy`: link files only.
- `scripts/setup/deploy.sh --dry-run`: preview links and conflicts.
- `scripts/setup/check.sh`: validate shell syntax and mapping sources.

Application installation, upgrades and icon overrides are separate explicit
commands. There is no CI. Verify with `check.sh` and shell syntax checks on
changed scripts. Test linker behavior in a temporary home, not the live home.

`update` uses `bin/lib/install-staleness.zsh` to track installed tool inputs.
Its records live in `~/.local/state/fzf-app-store/`. Preserve that independent
workflow when changing deployment.

## The src/ side: real code, real builds

`src/<tool>/` holds tool source (Rust: the AppKit menu bar widgets — `battery-widget`,
`cpu-usage-widget`, `free-disk-space-widget`, `menu-tidy` — plus `leaf`, `pimped`,
`neuroserver-select-preset`, `topaz-select-preset`; Go: `spill`, `iospeed`, `open-in-forklift`, `obsbot-rtsp-widget`,
`system-uptime-widget`; Swift: the `src/utils/` video ML CLIs `avinterp`,
`avupscale`, `avremove`). These are the only parts of the repo with a
build/install step and tests — Deployment does not build them. `src/utils/` is a
group directory like `src/jobs/`: each tool under it is its own Swift package
with its own `install-<name>.sh`.

New menu bar widgets go in Rust, against `objc2` directly — `battery-widget` and
`free-disk-space-widget` are the reference implementations. No wrapper library, no
vendored fork, no `.app` bundle, except where macOS permissions require one.
`job-folder` needs a bundle for notifications, so its installer *generates* the
`.app` into `~/Applications`. `wifi-widget` needs a bundle for Location access:
`src/wifi-widget/bundle.sh` generates `target/release/WiFi Widget.app` and
`scripts/install/install-wifi-widget.sh` copies it to `~/Applications`, opened
through LaunchServices, with Open at Login instead of a LaunchAgent. Nothing
bundled is ever checked in.

```bash
scripts/install/install-<name>.sh    # canonical build+install; most install to ~/.local/bin
cd src/leaf && cargo test        # Rust suites live in src/tests/ (leaf is the largest)
cd src/leaf && cargo test toc    # single test / filter
cd src/obsbot-rtsp-widget && go test ./...
```

### Two kinds of installer

`src/<tool>` builds from a tree inside this repo. Some tools are separate
repositories of mine instead — `switchblade`, `abner`, `tagform`, `chordpro-tui`, `safesync`, `motherfucker`, `mik-rs` — and
those clone into `$SRC_PATH` (`~/src` by default, exported from
`config/zsh/zshenv.zsh`). Their installers share one driver,
`scripts/install/lib/git-source-install.sh`, which on every run fetches,
fast-forwards **only** when upstream is strictly ahead, and rebuilds only when
the tree moved or the binary predates the checked-out commit. A dirty tree or a
local commit upstream doesn't have is never clobbered — it warns and builds what
is checked out. `FORCE=1` rebuilds regardless.

Adding another is three lines: source the driver and call
`git_source_install <name> <url> <cargo|go>`. Repos that ship
`packaging/build-app.sh` (switchblade, abner) use `app <AppName>` instead: the
install is `/Applications/<AppName>.app`, built by that script, and
`~/.local/bin/<name>` is a shim into the bundle's launcher.

Everything installs to `~/.local/bin`, including these — *not* `~/.cargo/bin` or
`~/go/bin`, which is where `cargo install` and `go install` would put them. One
location keeps the absolute paths in GUI-launched config honest.

Installers are named `scripts/install/install-<name>.sh` — follow that for new ones
(a few legacy scripts predate the prefix). Prefer the installer over a hand-rolled
`cargo install`/`go build` — it pins the install path the rest of the config expects
(e.g. `pimped` must be on PATH for the zsh precmd prompt hook in
`config/zsh/zshrc.zsh` to work).

**mik** (`$SRC_PATH/mik-rs`, `scripts/install/install-mik.sh`) is the one
git-source install with a second step: after the binary it builds the
*runtime*, a private copy of Mixed In Key 11.2.6's analysis code, from the
installed app into `<checkout>/runtime` (git-ignored, proprietary — never
commit or copy it anywhere). The binary finds that runtime by the checkout path
it was compiled from, so a moved checkout means re-running the installer.
`MIK_APP` points it at the app when Spotlight can't.

**safesync** lives at `$SRC_PATH/safesync` ([GitHub](https://github.com/monomadic/safesync))
and installs through `scripts/install/install-safesync.sh`.
**It is guarded by sentinels, not by care.** A drive takes part only if
it carries `.safesync/drive.toml` — `role = source | backup | scratch` pinned
to the volume UUID — and every command that writes media opens both sentinels
first: `sync` runs only source → the backup that names that source, `fill`
writes only onto scratch or unmarked disks, and a source is never written
except for its own index. The index (`.safesync/index-GENERATION.jsonl`) on
the drive is the source of truth; the copy in `~/Library/Application
Support/safesync/manifests/` is for `lookup` while the drive is unplugged.
Fingerprints are reused by file ID + size + mtime, so a rescan reads only new
files. Tests build real APFS ram disks (`tests/sync.rs`, ~25 s). It is meant
to replace `rclone-tower-safe`; until it has, the two coexist and neither
knows about the other's history directory. Don't add a journal, lease or
relationship layer back — that version was cut on purpose (git history before
2026-09-23).

**Topaz has two render backends.** Everything under `topaz-*` (the mpv `z`
menu, `topaz-encode`, `topaz-pick`, `topaz-workflow`, and the
`src/topaz-select-preset` TUI over them) drives the app's ffmpeg with a
`tvai_up` filter. The generative models (Starlight Precise, Astra,
Hyperion 2) are served by the app's separate `neuroserver` process instead and
are unreachable from that filter; presets for them declare `ns_model` /
`ns_store` / `ns_params` and are rendered by `topaz-preview-frame` (stills) and
by `src/neuroserver-select-preset` (the TUI, whose binary also contains the
whole-clip encoder — `neuroserver-encode` is a symlink to it, not a script).
Starlight *Mini* is not one of them: it is a three-part coreml model that
`tvai_up` loads itself. Don't add a neuroserver path to `topaz-encode`.
The encoder writes fragmented MP4/MOV on purpose: it is what lets the TUI show
live frames and what makes `--resume` possible.

**The jobs queue** is infrastructure other tools can build on: a workflow is
a folder in `~/jobs` with a `job.sh`, and a file dropped in its `input/` runs
through it (see `job-folder` below). Anything that needs "run this later / on
the server" should install a workflow and queue files with
`send-job [--job job.sh] <workflow> <files...>` instead of inventing its own
daemon — `topaz-job` and `interpolate-resolve-job` are the examples: each
generates a `job.sh` for its settings (one workflow per preset / fps) and hands
it and the inputs to `send-job`, which copies (hidden temp in the jobs root,
then an atomic `mv` into `input/`) or, with `--link`, symlinks.

Hand-written workflows are tracked as `config/jobs/<name>/job.sh`. The
Deployfile's `link_tree config/jobs "$HOME/jobs" …` links each one (and any
helper beside it) into a *real* `~/jobs/<name>/` folder and `ensure_dir`
creates `input/ output/ done/ failed/` there — so the script is versioned and
the job data never touches the repo. The ignore patterns on that `link_tree`
and the matching `.gitignore` rules keep queue folders and `*.log` out of both
deploy and git; keep them in step. `send-job --job` refuses to overwrite a
linked `job.sh`: edit the tracked one instead.

`src/jobs/` is a cargo workspace — the one nested directory under `src/` —
because its two crates share a lockfile, a target dir and pinned objc2
versions:

| | |
|---|---|
| `src/jobs/job-folder` | the queue: one folder per workflow (`job.sh` + `input/` + `output/`), runner and menu in one process. A `.app` with no agent — the queue runs while it is open |
| `src/jobs/job-core` | the drawing: menu rows, the menu bar icon, progress parsing |

The older `.job` system — `job-daemon` running jobs whose folder was their
state (`_ready`/`_running`/…), `job-monitor` watching it over SMB — was retired
on 2026-09-30; its source is in git history. Don't bring back `.job` files or
state directories. A **job folder** is any directory under `~/jobs` (or
`$JOBS_DIR`) holding a `job.sh`; the script is the workflow and the folder is
its queue. Drop files into `input/` and each runs through `job.sh`, oldest
first, one at a time per folder, with `$INPUT` (absolute path), `$INPUT_DIR`,
`$INPUT_FILE`, `$INPUT_NAME` (no extension) and `$OUTPUT_DIR` set, cwd the job
folder. stdout/stderr append to `stdout.log`/`stderr.log` under a per-run
header. The input stays in `input/` while it runs, then moves to `done/` or
`failed/` — so a restart re-queues whatever was left, and retry is dragging a
file back. Get files in by copying to `~/jobs` and then `mv`-ing into
`input/` (an atomic rename). Direct drops get light checks: no dotfiles or
`.part`-style names, size and mtime steady for `$JOB_SETTLE` seconds
(default 2), and not open for writing by this user's processes.
Running/held/paused state lives only in memory, so quitting stops the jobs, and
no second machine can watch the queue.

**`cleanup`** (`bin/cleanup`, zsh + fzf, vim keys) is the disk-space menu: sizes,
a risk colour and advice per row, submenus (`▸`) for Topaz models, large files and
`~/src`. Its rules: the Downloads → Tower sync only ever *adds* — nothing on the
Tower is overwritten or deleted, same-path files with a different size are listed
and left alone — and anything "moved" locally goes to the Trash after a size check on
the Tower, never `rm`. Repos with unpushed, uncommitted or remote-less work refuse
removal. Topaz rows come from `bin/lib/cleanup-topaz-models.py`, which follows
composite models (Starlight Mini) so their parts aren't reported as unused. Paths and
thresholds are `CLEANUP_*` env vars (listed in the script header). Test it against
temp dirs with `CLEANUP_SYNC_SRC/DST`, `CLEANUP_ARCHIVE`, `TOPAZ_APP` and
`CLEANUP_TOWER_UP=1` — never by answering its prompts with the live paths.
**Coding agents commit their own work.** `bin/agent-commit-guard` is the Stop
hook for Claude Code (`config/claude/settings.json`) and Codex
(`config/codex/hooks.json`): a turn can't end on a dirty tree, and the commit
carries a `Claude-Session:` / `Codex-Session:` trailer. Behind it,
`bin/agent-snapshot` (LaunchAgent from
`scripts/install/install-agent-snapshot.sh`, every 5 min) saves each dirty
worktree under `$SRC_PATH` and this repo to `refs/wip/<branch>` without
touching the branch, index or files, naming the sessions that were active
there. Its untracked-file limits (junk dirs, media, per-file and per-snapshot
size) exist because an unignored venv once put 768 MB into a repo's `.git` —
keep them.

## Recipe: add config for a new tool

1. Create `config/<tool>/` — flat, named after the tool.
2. Add `link_file` or `link_tree` calls to `Deployfile`.
3. Run `scripts/setup/check.sh`, then `scripts/setup/deploy.sh`.

## Where things go

| Path | Purpose |
|---|---|
| `config/<tool>/` | active config source, one tool per directory, flat |
| `config/zsh/` | shell config only — rc files, `autoload/`, `completions/`. No commands live here any more |
| `bin/` | **every** user-facing command, one flat directory (→ per-file symlinks in `~/.local/bin/`) |
| `bin/lib/` | the one exception to a flat `bin/` — sourceable snippets and preset data, not commands |
| `scripts/` | subdirectories only; nothing loose at the top level |
| `scripts/setup/` | bootstrap, deploy, and health-check entrypoints |
| `scripts/install/` | `install-<name>.sh` build+install scripts for `src/` |
| `scripts/tweaks/` | one-shot macOS `defaults write` tweaks — never run by deploy |
| `Deployfile` | plain zsh deployment calls |
| `src/<tool>/` | small personal utility source trees (Rust for the menu bar widgets and `leaf`, Go for the rest) — build via `scripts/install/install-<name>.sh` |
| `$SRC_PATH` (default `~/src`) | checkouts of *separate* upstream repos, cloned and kept current by their installers. Outside this repo on purpose |
| `assets/` | fonts, icons, and colour LUTs (`assets/LUTs/` deploys into Resolve and Final Cut) |
| `vendor/bin/` | retained third-party binaries; `bin/` holds the thin `exec` shim for each |
| `_quarantine/` | commands dropped from PATH but kept in history. Never referenced, never deployed, never added to |

## Rules that prevent rework

- **No new domain buckets under `config/`** (`editors/`, `media/`, `windowing/`...). A few legacy ones exist; don't add files to them — use `config/<tool>/`.
- **`bin/` is the only home for commands.** There is no second command directory — the old `bin/` vs `config/zsh/bin/` split is gone, and so is the rule about picking between them. A new command goes in `bin/`, whatever language it is in.
- `bin/` deploys as one symlink **per file** into `~/.local/bin/`, which also holds binaries the `scripts/install/` scripts build. So a new command must not collide with an installed binary name (`pimped`, `leaf`, the widgets, pipx/uv shims) — the linker skips conflicting files, reports them, and continues.
- Retiring a command means `git mv bin/<cmd> _quarantine/bin/`, not deleting it, and removing every reference first. Nothing in `_quarantine/` may be referenced from live config.
- Executables meant to be invoked as commands are **extensionless**. Use `.zsh`/`.sh`/`.py` only for sourced or clearly single-language utilities.
- Local config templates are checked in as `*.example`; the live file is gitignored.
- Helix is the active editor. `config/neovim/` is dormant source — keep it out of active profiles.
- `Brewfile` is bootstrap-critical: only add things the shell/editor/config actually need. Large apps with no config dependency go in `Brewfile.optional`, which bootstrap never installs.
- No secrets, credentials, installers, `.app` bundles, or large binaries in the tree. `.env` is gitignored; `.env.example` documents expected vars.
- Commit messages: short imperative subject line ("Add ytq pop command", "Refactor Zellij config for Yazi-first sessions").
