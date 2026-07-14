# Zsh Config — Outstanding Tasks

P0 (leak scrub), P1–P3 (startup 109 ms → 27 ms, keybinding/history conflicts, dead
aliases), the footgun-alias cleanup, and the P4 `alias.zsh` re-home are all done and
pushed. Nothing is outstanding. The two remaining P4 ideas were considered and
deliberately left alone — rationale kept here so they don't get re-raised:

## Considered and declined

### Slim `zshenv.zsh` — not worth it

The idea was to stop interactive-only exports (media glob arrays, `JUMP_DIRS`, FZF
options) from loading for every non-interactive zsh. But **zero `bin/` scripts read any
of them**, and startup is already 27 ms, so moving them is cosmetic with a small risk
of breaking a subshell that relies on the export.
(If ever touched: `ZSH_SCRIPT_PATHS` is declared as an exported array — zsh can't export
arrays, so it silently exports a scalar. `configure-bin` reads it. That's the one real
bug in zshenv worth fixing.)

### Rename `autoload/` — skip

Nothing in it is fpath-`autoload`ed; it's all sourced, so the name is a misnomer. But
renaming is cosmetic, forces a `dotter` mapping change plus a redeploy that re-points a
live symlink (`~/.zsh/autoload`), and risks leaving a dangling old symlink. Risk > value.
