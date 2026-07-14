# Zsh Config — Outstanding Tasks

The P1–P3 assessment (startup 109 ms → 27 ms, keybinding/history conflicts, dead
aliases) was applied and pushed 2026-07-13. The full history and rationale live in
git (`git log --oneline --grep zsh`). What remains:

## P0 — leak cleanup (manual, off-repo)

The Brave profile copies were purged from git history and force-pushed; the repo tree
and history verify clean and there are no forks. Still on you:

- [ ] **Rotate sessions** for any site used with that profile (Ultimate Guitar at
      minimum) — treat those cookies/logins as compromised.
- [ ] **Sync other machines.** A clone pushed `a27f536b` recently and still holds the
      leaked history; run `git fetch && git reset --hard origin/master` there (or
      re-clone). A `push --force` from a stale clone would resurrect the leak.
- [ ] **Delete on-disk profile dirs** when ready — they remain, gitignored:
      `config/zsh/.ug-browser-profile`, `config/zsh/bin/.ug-browser-profile`,
      `config/zsh/bin/songs/.ug-browser-profile`.
- [ ] **Delete the backup bundle** `~/config-pre-scrub-20260713.bundle` once
      confident — it still contains the leaked history.
- [ ] *(optional)* Ask GitHub Support to purge cached commit views; old SHAs stay
      fetchable until GC.

## P4 — structure (deferred; each its own reviewed change)

- [ ] **Re-home `alias.zsh` (770 lines).** Its yt-dlp / ffmpeg / media / kitty / rsync
      sections duplicate the domain files that already exist for those tools. Move each
      section into its module; keep `alias.zsh` for cross-cutting shortcuts (git, cargo,
      ls).
- [ ] **Slim `zshenv.zsh`.** Media glob arrays, `JUMP_DIRS`, and FZF options load for
      every non-interactive zsh (scripts, `zsh -c`, editor tooling). Keep zshenv to
      PATH/EDITOR/LANG/XDG + vars scripts need; move interactive-only exports into zshrc
      or the relevant module. Note `ZSH_SCRIPT_PATHS` is an exported array — zsh can't
      export arrays, so it silently exports a scalar.
- [ ] **Rename `autoload/`.** Nothing in it is `autoload`ed via fpath — it's all
      sourced. Rename to `rc.d/` or `modules/` for honesty. (True fpath autoloading
      isn't worth the churn at 27 ms startup.)

## Follow-ups noted but left alone (user's call)

- Footgun aliases that work but read wrong: `ga`/`gca` (silent `--amend`), `dd`
  (shadows coreutils), `~=grep` (breaks `~` autocd).
- `config/zsh/songs/` and `config/zsh/lua/module.lua` aren't zsh config — relocate or
  delete per the one-tool-per-dir repo rule.
