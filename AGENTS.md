# AGENTS.md

Personal dotfiles repo, deployed with [Dotter](https://github.com/SuperCuber/dotter).
**macOS only** — Linux config (i3, sway, waybar, foot, weston, refind, ...) has been removed; don't add it back.
Full layout rules: [docs/STRUCTURE.md](docs/STRUCTURE.md). Bootstrap docs: [README.md](README.md).

`CLAUDE.md` is a symlink to this file — edit `AGENTS.md`, never the symlink.

## The one thing to understand first

Dotter **symlinks** repo files into place (`default_target_type = "symbolic"`).
`config/yazi/` IS `~/.config/yazi/` on this machine. Editing a file in this repo
changes the live config immediately — no build, no deploy. Treat edits to `config/`
as live changes, and don't "install" anything by copying files out of the repo.

A deploy run is only needed when the *mapping* changes: a new package, a new
file entry in `dotter/global.toml`, or a changed target path.

## The repo is location-independent

Nothing assumes `~/config`. `scripts/**` scripts resolve the root from their own
path (`dirname $0/../..`), `bin/dotter-deploy` resolves through its
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

Two manifests, one syntax:

- `dotter/global.toml` — every package as a `[<name>.files]` section mapping repo path → target path. Alphabetical, grouped by purpose. Use the long `{ target = ..., type = ..., recurse = ... }` form only when overriding defaults.
- `dotter/local.toml` — **gitignored**; this machine's package selection plus variable overrides. Never expect it in git; never commit it.
- `dotter/local.toml.example` — the single tracked template (there is no longer a per-platform set); optional packages stay listed but commented out so it doubles as an inventory.
- Some `config/<tool>/` dirs are intentionally source-only (not wired into Dotter yet): beatportdl, compressor, git, homebrew, iterm, ollama, python, tag-media.

## Commands

```bash
scripts/setup/bootstrap.sh      # full machine setup: CLT, Homebrew, Brewfile, clone, deploy
scripts/setup/check.sh          # preflight: validates manifests, source paths, package names
scripts/setup/packages.sh list  # show every package and whether it is on for this machine
scripts/setup/packages.sh enable <name>   # edit local.toml without hand-syncing lists
scripts/setup/deploy.sh         # run Dotter deploy (wraps bin/dotter-deploy)
scripts/setup/deploy.sh --upgrade # syncs Yazi packages, repairs yt-dlp, brew upgrade, rebuilds stale src/ tools
scripts/setup/deploy.sh --icons   # reapply macOS app icon overrides
scripts/setup/deploy.sh --full    # --icons plus --upgrade
```

A bare deploy only symlinks config — that is the fast, safe default. `--upgrade`
is the one that touches the outside world, and each of its steps has an escape
hatch: `DOTTER_SKIP_YAZI_PACKAGES=1`, `DOTTER_SKIP_YTDLP=1`, `DOTTER_SKIP_BREW=1`,
`DOTTER_SKIP_SRC=1`. Every step is non-fatal — an unreachable tap or a failed
build warns and the run continues, because the job that matters is the symlinking.

The `src/` rebuild step is deliberately narrow: it only rebuilds a tool that is
**already installed** on this machine (a deploy is the wrong place to acquire new
software) and only when a real build input — `*.rs`, `*.go`, `Cargo.toml`,
`go.mod`, or the installer itself — changed since the install. READMEs
and design mockups don't count, or every doc edit would rebuild the world. Add a
new tool to the `specs` table in `bin/dotter-deploy` when it gets an installer.

"Changed" is decided by `bin/lib/install-staleness.zsh`, shared with
`fzf-app-store` (the browser over `scripts/install/`): a content hash recorded in
`~/.local/state/fzf-app-store/` after each install, falling back to git history
when there is no record. Never raw mtimes — a `git mv` resets them and makes
every installed tool look stale.

`deploy.sh` runs `check.sh` automatically unless `DOTTER_SKIP_HEALTHCHECK=1`.
There is no CI. For config and script changes, verification = `check.sh` passing,
plus `zsh -n` / `bash -n` on any shell script you touched.

## The src/ side: real code, real builds

`src/<tool>/` holds tool source (Rust: the AppKit menu bar widgets — `battery-widget`,
`cpu-usage-widget`, `free-disk-space-widget`, `menu-tidy` — plus `leaf`, `pimped`,
`motherfucker`, `neuroserver-select-preset`, `topaz-select-preset`; Go: `spill`, `iospeed`, `open-in-forklift`, `obsbot-rtsp-widget`,
`system-uptime-widget`). These are the only parts of the repo with a
build/install step and tests — Dotter does not touch them.

New menu bar widgets go in Rust, against `objc2` directly — `battery-widget` and
`free-disk-space-widget` are the reference implementations. No wrapper library, no
vendored fork, no `.app` bundle. `job-monitor` is the one exception on the bundle
rule: it needs one for notifications, so its installer *generates* the `.app` into
`~/Applications` — nothing bundled is ever checked in.

```bash
scripts/install/install-<name>.sh    # canonical build+install; most install to ~/.local/bin
cd src/leaf && cargo test        # Rust suites live in src/tests/ (leaf is the largest)
cd src/leaf && cargo test toc    # single test / filter
cd src/obsbot-rtsp-widget && go test ./...
```

### Two kinds of installer

`src/<tool>` builds from a tree inside this repo. Some tools are separate
repositories of mine instead — `switchblade`, `tagform`, `chordpro-tui` — and
those clone into `$SRC_PATH` (`~/src` by default, exported from
`config/zsh/zshenv.zsh`). Their installers share one driver,
`scripts/install/lib/git-source-install.sh`, which on every run fetches,
fast-forwards **only** when upstream is strictly ahead, and rebuilds only when
the tree moved or the binary predates the checked-out commit. A dirty tree or a
local commit upstream doesn't have is never clobbered — it warns and builds what
is checked out. `FORCE=1` rebuilds regardless.

Adding another is three lines: source the driver and call
`git_source_install <name> <url> <cargo|go>`. Add the name to the loop in
`upgrade_git_source_tools` in `bin/dotter-deploy` so `--upgrade` keeps it current.

Everything installs to `~/.local/bin`, including these — *not* `~/.cargo/bin` or
`~/go/bin`, which is where `cargo install` and `go install` would put them. One
location keeps the absolute paths in GUI-launched config honest.

Installers are named `scripts/install/install-<name>.sh` — follow that for new ones
(a few legacy scripts predate the prefix). Prefer the installer over a hand-rolled
`cargo install`/`go build` — it pins the install path the rest of the config expects
(e.g. `pimped` must be on PATH for the zsh precmd prompt hook in
`config/zsh/zshrc.zsh` to work).

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

**The jobs queue** is infrastructure other tools can build on: drop a
`TARGET.job` shell script into `~/jobs` and it runs. Anything that needs "run
this later / on the server" should write a `.job` file (or ship one with
`send-job`) instead of inventing its own daemon.

**A job is a folder, and the folder it sits in is its state** — `_ready`,
`_running`, `_paused`, `_ok`, `_failed`. There is no lock file, no pause flag
and no status protocol, which means moving a folder is also how the queue is
*commanded*: drag a running job to `_paused` and the runner stops its process
group, drag it back and it resumes, drag it to `_failed` and it is terminated.
That works from Finder or from another machine over SMB, and the filesystem's
own permissions are the access control. Contract details:
`src/jobs/job-daemon/README.md`.

`src/jobs/` is a cargo workspace — the one nested directory under `src/`,
because these crates share a lockfile, a target dir and pinned objc2 versions,
and because `job-monitor` must *not* depend on the job loop:

| | |
|---|---|
| `src/jobs/job-daemon` | the only thing that runs jobs *for the folder protocol*. `--once` under a launchd WatchPaths trigger, or resident |
| `src/jobs/job-monitor` | the menu bar UI, for the local queue and for folders mounted over SMB. A normal `.app` you launch and quit; links no runner, but commands the queue by moving folders |
| `src/jobs/job-folder` | the variation: runner and menu in one process, queue held in memory. A `.app` with no agent — the queue runs while it is open |
| `src/jobs/job-core` | the shared library — the model, the filesystem observer, the rows and the icon |

In the daemon/monitor pair the UI is a client, never a runner: a crate with no
job loop linked into it cannot claim a job however it is launched, and it
doesn't need to, because every command is a folder move. Any number of UIs can
watch one queue, locally or across the LAN.

`job-folder` deliberately makes the opposite trade, for the machine you are
sitting at: one process runs the jobs and draws the menu, so the queue is a
`Vec<Job>` behind a mutex, pause is a `SIGSTOP` on the way back from the click,
and reordering is a splice. It keeps the `.job` drop folder (so `send-job` and
`topaz-job` are unchanged) and stages payloads through `ready/` → `done/`, but
writes no `.status` and no state directories — which is also why nothing can
watch it from another machine, and why quitting stops the jobs. Run it *or*
`job-daemon` on a given folder, never both.

## Recipe: add config for a new tool

1. Create `config/<tool>/` — flat, named after the tool itself (`config/helix`, not `config/editors/helix`).
2. Add a `[<tool>.files]` section to `dotter/global.toml`, in the right alphabetical spot within its group.
3. Add `"<tool>"` to `dotter/local.toml.example` (commented out unless it should be on by default).
4. `scripts/setup/packages.sh enable <tool>` if it should be active here, then `scripts/setup/deploy.sh`.

`check.sh` fails on manifest entries pointing at missing repo paths and on
`local.toml` selecting packages that don't exist in `global.toml`.

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
| `dotter/` | deployment manifests only |
| `src/<tool>/` | small personal utility source trees (Rust for the menu bar widgets and `leaf`, Go for the rest) — build via `scripts/install/install-<name>.sh` |
| `$SRC_PATH` (default `~/src`) | checkouts of *separate* upstream repos, cloned and kept current by their installers. Outside this repo on purpose |
| `assets/` | fonts, icons, and colour LUTs (`assets/LUTs/` deploys into Resolve and Final Cut) |
| `vendor/bin/` | retained third-party binaries; `bin/` holds the thin `exec` shim for each |
| `_quarantine/` | commands dropped from PATH but kept in history. Never referenced, never deployed, never added to |

## Rules that prevent rework

- **No new domain buckets under `config/`** (`editors/`, `media/`, `windowing/`...). A few legacy ones exist; don't add files to them — use `config/<tool>/`.
- **`bin/` is the only home for commands.** There is no second command directory — the old `bin/` vs `config/zsh/bin/` split is gone, and so is the rule about picking between them. A new command goes in `bin/`, whatever language it is in.
- `bin/` deploys as one symlink **per file** into `~/.local/bin/`, which also holds binaries the `scripts/install/` scripts build. So a new command must not collide with an installed binary name (`pimped`, `leaf`, the widgets, pipx/uv shims) — Dotter refuses to overwrite an unmanaged file and the deploy fails.
- Retiring a command means `git mv bin/<cmd> _quarantine/bin/`, not deleting it, and removing every reference first. Nothing in `_quarantine/` may be referenced from live config.
- Executables meant to be invoked as commands are **extensionless**. Use `.zsh`/`.sh`/`.py` only for sourced or clearly single-language utilities.
- Local config templates are checked in as `*.example`; the live file is gitignored.
- Helix is the active editor. `config/neovim/` is dormant source — keep it out of active profiles.
- `Brewfile` is bootstrap-critical: only add things the shell/editor/config actually need. Large apps with no config dependency go in `Brewfile.optional`, which bootstrap never installs.
- No secrets, credentials, installers, `.app` bundles, or large binaries in the tree. `.env` is gitignored; `.env.example` documents expected vars.
- Commit messages: short imperative subject line ("Add ytq pop command", "Refactor Zellij config for Yazi-first sessions").
