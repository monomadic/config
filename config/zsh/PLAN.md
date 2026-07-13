# Zsh Config — Assessment & Improvement Plan

Assessed 2026-07-13. Startup numbers measured on this machine with `hyperfine`
(20 runs, warmed) and a timestamped `xtrace` trace of a full interactive startup.

## Status — applied 2026-07-13

- **P0 done (repo side):** all three profile copies (`config/zsh/.ug-browser-profile`,
  `config/zsh/bin/.ug-browser-profile`, `config/zsh/bin/songs/.ug-browser-profile` —
  1494 files total) untracked, gitignored, purged from history with `git filter-repo`,
  and force-pushed. Remote tree and full history verified clean; the repo has **no
  forks**. Pre-scrub backup: `~/config-pre-scrub-20260713.bundle`.
  **Still on you:** (1) rotate sessions for sites used with that profile
  (Ultimate Guitar at minimum); (2) optionally ask GitHub Support to purge cached
  commit views; (3) any other machine with a clone (one pushed `a27f536b` recently)
  must re-clone or `git fetch && git reset --hard origin/master` — a habitual
  `push --force` from an old clone would resurrect the leak; (4) delete the on-disk
  profile dirs when ready (they remain, gitignored).
- **P1 done:** measured `zsh -i -c exit` **109 ms → 27 ms**. Extra finding while
  applying: the stray second compinit was accidentally load-bearing — it ran after
  `brew shellenv` and was what registered Homebrew site-functions completions
  (`_yt-dlp`, `_atuin`, …). Fixed properly by adding site-functions to the fpath
  block before the single compinit.
- **P2 done:** fzf owns `^R`/`^T` (dead custom widget removed, tv's `^R` rebind is
  overridden by keybindings.zsh, `^D` restored to default), history options fixed,
  prompt-middle reduced to `_magic-enter`, dormant `python.zsh`/`starship.zsh`/
  `vi-mode.zsh` deleted.
- **P3 done:** broken/duplicate aliases fixed or removed as listed below;
  `index.zsh` reduced to its live helpers; stray `gh` file deleted. `songs/` and
  `lua/` left in place — relocation is a user call.
- **P4 deferred:** alias.zsh re-homing, zshenv slimming, `autoload/` rename — larger
  churn, best done as its own reviewed change.

## TL;DR

| Item | Impact |
|---|---|
| P0: Brave browser profile (Cookies, Login Data) tracked in a **public** repo | security incident — scrub + rotate |
| P1a: stray second `compinit` in zshrc | −30 ms/startup |
| P1b: `op completion zsh` spawned every startup | −40 ms/startup |
| P1c: `tv init zsh` runs **twice**, uncached | −10 ms/startup |
| P1 combined (measured) | **109 ms → 30 ms** (`zsh -i -c exit`) |
| P2: three different things fight over `^R`/`^T`; custom history widget is dead code | ergonomics/correctness |
| P3: ~10 broken aliases, duplicate definitions, dormant modules | hygiene |

---

## P0 — SECURITY: browser profile committed to a public repo

`config/zsh/.ug-browser-profile/` is **tracked in git** (253 files, ~11 MB) and the
repo (`github.com:monomadic/config`) is **public**. It contains a full Brave/Chromium
profile including:

- `Default/Cookies` + `Cookies-journal` — session cookie database
- `Default/Login Data` — saved-password database
- `Default/Account Web Data`, `BraveWallet/`, autofill databases, cache

Chromium encrypts cookie values/passwords with a key held in the macOS Keychain, so
the DBs are not trivially decryptable off-machine — but hostnames, accounts, and any
legacy-unencrypted values leak, and it's an 11 MB junk payload regardless. Nothing in
the repo references this directory (it was presumably a `--user-data-dir` for an
Ultimate Guitar scraping session, committed by accident in commit `1e1a6f7e "update:"`).

Remediation (in order):

1. Log out / invalidate sessions for any site used with that profile (Ultimate Guitar
   at minimum); treat those cookies as compromised.
2. Remove from the working tree and index:
   ```sh
   git rm -r --cached "config/zsh/.ug-browser-profile"
   rm -rf config/zsh/.ug-browser-profile        # nothing uses it
   echo ".ug-browser-profile/" >> .gitignore
   ```
3. Scrub history (rewrites all hashes; requires force-push):
   ```sh
   git filter-repo --invert-paths --path config/zsh/.ug-browser-profile/
   git push --force origin master
   ```
4. Old commits stay retrievable on GitHub until GC — contact GitHub Support to purge
   cached views, and check for forks (`gh api repos/monomadic/config/forks`).

Also delete the stray 1-byte tracked file `config/zsh/gh` while here.

---

## P1 — Startup time

Measured baseline: `zsh -i -c exit` = **109 ms** (plain `zsh -c exit` = 3.6 ms, so
~105 ms is zshrc). The lazy first-prompt hooks (fzf sourcing + cached starship) add
**~22 ms** on top, and inside kitty the `_kitty_sync_shell_title` call adds a
`git rev-parse` + two `kitty @` socket round-trips before the first prompt.

### P1a. Delete the stray second `compinit` (109 → 80 ms)

[zshrc.zsh:177](config/zsh/zshrc.zsh:177):

```zsh
fpath+=~/.zfunc; autoload -Uz compinit; compinit
```

This is installer cruft (poetry/rustup-style). `~/.zfunc` **does not exist**, and the
plain `compinit` re-runs the full security audit (`compaudit` stats every file in
every fpath dir) and maintains a *second* dump at `~/.zcompdump`, duplicating the
carefully cached `compinit -C -d $COMPDUMP` at line 29. The xtrace trace attributes
~88 % of traced startup to compinit/compdef/compaudit/compdump.

Fix: delete the line; `rm ~/.zcompdump*`. If `~/.zfunc` is ever needed, add it to the
`fpath=( ... )` block at the top *before* the single compinit.

### P1b. Cache `op completion zsh` (−40 ms)

[completions.zsh:8](config/zsh/autoload/completions.zsh:8) spawns the 1Password CLI
every startup — the single slowest external command in the trace. Use the same
mtime-invalidated cache pattern already used for brew/starship:

```zsh
_op_init="$ZSH_CACHE_DIR/op-completion.zsh"
if [[ ! -s $_op_init || $(command -v op) -nt $_op_init ]]; then
  command op completion zsh >! "$_op_init"
fi
source "$_op_init"
```

### P1c. `tv init zsh` runs twice — dedupe and cache (−10 ms)

It is eval'd in **both** [completions.zsh:11](config/zsh/autoload/completions.zsh:11)
and [zshrc.zsh:165](config/zsh/zshrc.zsh:165). Keep one, cached like the above.
Note: `tv init` also does `bindkey '^R' tv-shell-history` — see P2 before deciding
where it lives.

> Measured result of P1a+b+c together: **109 ms → 30 ms** (3.7×).

### P1d. `export HOSTNAME=$(hostname)` in zshenv

[zshenv.zsh:104](config/zsh/zshenv.zsh:104) forks `hostname` for **every** zsh
invocation, including every script and subshell. zsh already provides `$HOST`:

```zsh
export HOSTNAME=$HOST
```

### P1e. Completion-dump freshness (removes a workaround)

The cached dump `~/.cache/zsh/.zcompdump-5.9` dates from **Aug 2025**; `compinit -C`
never revalidates it, which is why [completions.zsh:28-40](config/zsh/autoload/completions.zsh:28)
carries a manual `compdef` list for "completions the cached compinit misses".
Replace both with an mtime check — rebuild the dump only when a completion dir is
newer than it:

```zsh
autoload -Uz compinit
typeset -g COMPDUMP="$ZSH_CACHE_DIR/.zcompdump-$ZSH_VERSION"
if [[ -n $COMPDUMP(#qN.mh-24) || $ZSH_CONFIG_DIR/completions -nt $COMPDUMP ]]; then
  compinit -d "$COMPDUMP" -C -i     # fresh enough: skip audit
else
  compinit -d "$COMPDUMP" -i        # stale: full (re)build
  zcompile -U -- "$COMPDUMP"
fi
```

(Any recipe is fine as long as a full `compinit` runs when
`$ZSH_CONFIG_DIR/completions` or `/opt/homebrew/share/zsh/site-functions` changes;
then delete the manual `compdef` block.)

### P1f. Optional / marginal

- `zcompile` the big sourced files (`alias.zsh`, `function.zsh`, `ffmpeg.zsh`) — sub-ms
  each in the trace; only worth it as a deploy-hook one-liner.
- zshrc lines 167–173 add LM Studio to PATH **three times**, and `init-path` already
  adds both entries; `typeset -U path` hides the dupes but the lines are dead — delete.
- `_kitty_sync_shell_title` runs `git rev-parse` + up to two `kitty @` calls on every
  `chpwd` and at startup. If prompt-to-prompt cd latency ever bothers you, run it
  as `( ... & )` detached — output is discarded already, nothing reads its result.

---

## P2 — Keybinding ownership & UI

### The `^R` / `^T` race

Startup order today:

1. `keybindings.zsh` (config_files loop): binds `^R`→`fzf-history-widget` (custom,
   defined in the same file), `^T`→`fzf-file-widget`, then immediately re-binds
   `^t`→`fzf-kitty-switch-tabs` — same key, so the file-widget bind is dead on arrival.
2. `zshrc:165` `tv init` then re-binds `^R`→`tv-shell-history`.
3. **First prompt**: `_lazy_fzf_precmd` sources fzf's official `key-bindings.zsh`,
   which *redefines the function* `fzf-history-widget` (same name as the custom one)
   and re-binds `^R`→`fzf-history-widget`, `^T`→`fzf-file-widget`, `Alt-C`.

Net effect:

- The custom `fzf-history-widget` in keybindings.zsh **never runs** — fzf's official
  widget replaces it at the first prompt. Cmd+Shift+H also ends up on the official one.
- `^R` is `tv-shell-history` for the first prompt only, then silently becomes fzf
  history. Pick one owner.
- `fzf-kitty-switch-tabs` has **no function definition anywhere** — pressing `^T`
  before the first prompt errors ("no such widget"), and after the first prompt fzf
  owns `^T` anyway. Delete the bind or implement the widget.

Recommendations:

- Rename the custom widget (e.g. `my-history-widget`) if you want to keep it, or
  delete it and let fzf's own `^R` widget be the single owner (it already gets
  `FZF_DEFAULT_OPTS`).
- Decide `^R`'s owner once (fzf *or* tv), and remove the loser's bind right after
  its init line.
- Move all binds that fzf clobbers (`^R`, `^T`, `Alt-C`) *into* the lazy loader —
  i.e. have `_lazy_fzf_precmd` source `keybindings.zsh` last — so user binds always
  win regardless of load order.

### Other binding issues

- `bindkey '^D' fzf-dir-widget` ([keybindings.zsh:254](config/zsh/autoload/keybindings.zsh:254)):
  kills Ctrl+D EOF-exit, and the widget itself just pipes `fd | fzf | tr` to stdout —
  it never touches `LBUFFER`, so the selection is printed onto the screen instead of
  inserted. Either make it a real widget (`LBUFFER+=...`) or drop it.
- `vi-mode.zsh` and `starship.zsh` and `python.zsh` are not in `config_files` —
  dormant. `python.zsh` would break anyway (`pyenv` is not installed). Delete or move
  to an `inactive/` note; per repo rules dormant configs shouldn't sit next to live ones.

### Prompt stack (three layers, two stale)

- [prompt.zsh](config/zsh/autoload/prompt.zsh) sets `PROMPT`, but lazy starship
  replaces it at the first precmd — the prompt is never seen. Keep it only as an
  explicit no-starship fallback, else delete.
- [prompt-middle.zsh](config/zsh/autoload/prompt-middle.zsh) captures `PS1o="$PS1"`
  at source time (pre-starship, so stale) and computes `halfpage` from `$LINES` once
  (stale after any resize). Only `_magic-enter` (Shift+Enter) is actually used —
  keep that, recompute `LINES/2` inside the widget (it already does), and delete
  `prompt_middle`/`prompt_restore`/`PS1o`/the terminfo loops.

### History options ([history.zsh](config/zsh/autoload/history.zsh))

- `SAVEHIST=10000` < `HISTSIZE=100000` — you trim 90 % of history on write; probably
  want both 100000.
- `SHARE_HISTORY` + `INC_APPEND_HISTORY` + `INC_APPEND_HISTORY_TIME` are mutually
  exclusive by design — zsh warns that setting more than one is wrong. Keep
  `SHARE_HISTORY` only (it implies incremental append).
- `INC_APPEND_HISTORY` is set twice (lines 29, 38).

### Footgun aliases (deliberate? worth a second look)

- `alias ga="git add . && git commit --amend --no-edit"` — "ga" reads as *git add*
  but silently amends the last commit.
- `alias dd='dotter-deploy'` — shadows coreutils `dd` interactively.
- `alias ~=grep` — breaks `~` + autocd to `$HOME` (alias wins over autocd).
- `alias rm="rm -i"` — fine, but note scripts don't see it; consider `rip` given the
  destructive-move history in this setup.

---

## P3 — Dead code & broken references

Aliases/functions that reference things that don't exist anywhere (autoload/, bin/):

| Where | Broken piece |
|---|---|
| [media.zsh:172](config/zsh/autoload/media.zsh:172) | `@@` → `.play-sort` (undefined) |
| [media.zsh:185](config/zsh/autoload/media.zsh:185) | `@external` → `@volumes` (undefined) |
| [media.zsh:149](config/zsh/autoload/media.zsh:149) | `.select-clips` → `fd-clips` (undefined) |
| [media.zsh:116](config/zsh/autoload/media.zsh:116) | `.select-pwd-sorted` → `.ls-pwd-sorted` (undefined; `.ls-sorted-pwd` exists) |
| [alias.zsh:440](config/zsh/autoload/alias.zsh:440) | `mpv-play-porn` → `$~MEDIA_GLOBS` — never defined (`ADULT_GLOBS` is) |
| [media.zsh:173](config/zsh/autoload/media.zsh:173) | `@@@` → same undefined `MEDIA_GLOBS` |
| [index.zsh:60-72](config/zsh/autoload/index.zsh:60) | `index-play-top` / `index-play-checked` / `index-play-checked-top` call `index-cat`/`index-cat-checked`/`index-grep-top`, all commented out above |

Duplicate definitions (later one silently wins):

- `mpv-play-local` defined twice back-to-back ([index.zsh:7-15](config/zsh/autoload/index.zsh:7))
- `.select` twice, identical ([media.zsh:69](config/zsh/autoload/media.zsh:69), [113](config/zsh/autoload/media.zsh:113))
- `.play-tower-downloads` twice with **different** commands ([media.zsh:76](config/zsh/autoload/media.zsh:76), [166](config/zsh/autoload/media.zsh:166))
- `.play-tower-masters` twice with **different** commands ([media.zsh:77](config/zsh/autoload/media.zsh:77), [164](config/zsh/autoload/media.zsh:164))
- `.play-local` set in index.zsh, overwritten by media.zsh:74
- `FZF_COMPLETION_TRIGGER` exported in both zshenv.zsh:122 and fzf.zsh:5

Stale/stray:

- `.config-env` → `scripts/autoload/alias.zsh` — path doesn't exist (file is `autoload/alias.zsh`)
- `config/zsh/songs/`, `config/zsh/lua/module.lua`, 1-byte `config/zsh/gh` — not zsh
  config; relocate or delete (repo rule: one tool per directory, no stray buckets)

---

## P4 — Structure

- **`autoload/` is a misnomer**: every file in it is *sourced* eagerly (or lazily for
  fzf), not `autoload`ed via fpath. Either rename to `rc.d/`/`modules/` for honesty,
  or actually convert the pure-function files to fpath autoloading. Given P1 fixes
  bring startup to ~30 ms, true autoload conversion is **not worth the churn** —
  a rename plus the cleanups above is the pragmatic move.
- **`alias.zsh` (770 lines) duplicates the domain files**: it contains yt-dlp,
  ffmpeg, media, kitty, and rsync sections while `yt-dlp.zsh`, `ffmpeg.zsh`,
  `media.zsh`, `kitty.zsh`, `rsync.zsh` exist for exactly those domains. Re-home the
  sections; keep `alias.zsh` for genuinely cross-cutting shortcuts (git, cargo, ls).
- **zshenv is doing zshrc's job**: media glob arrays, `JUMP_DIRS`, FZF options, and
  friends load for every non-interactive zsh (scripts, `zsh -c`, editor tooling).
  Keep zshenv to PATH/EDITOR/LANG/XDG + variables scripts actually need; move the
  interactive-only exports into zshrc or the relevant module.
- Minor: `unset config_files config_file` after the source loop; `ZSH_SCRIPT_PATHS`
  is an exported array (zsh can't export arrays — it silently exports a scalar).

---

## Suggested order of execution

1. **P0** — scrub browser profile + rotate sessions (do first; history rewrite is
   easier before new commits pile up).
2. **P1a–d** — the four startup fixes (measured: 109 → 30 ms; each is a small,
   independent diff; verify with `hyperfine 'zsh -i -c exit'` after each).
3. **P1e** — compdump freshness + delete the manual compdef workaround.
4. **P2** — pick owners for `^R`/`^T`, delete the dead custom widget and
   `fzf-kitty-switch-tabs`, fix `^D`, fix history options, consolidate prompt files.
5. **P3** — delete/fix broken and duplicate aliases (grep list above).
6. **P4** — re-home alias.zsh sections, slim zshenv, rename `autoload/` (optional).

Verification for every step: `zsh -n` on touched files, `setup/macos/check.sh`, open
a new kitty tab and try: Tab-completion, `^R`, `^O`, Ctrl+Space (yazi), Shift+Enter.
