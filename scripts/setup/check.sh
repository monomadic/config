#!/bin/zsh -f
# Validate mapping syntax and sources without changing anything.
ROOT="${${0:A}:h:h:h}"
exec /bin/zsh -f "$ROOT/scripts/setup/link.zsh" --check
