#!/bin/bash
# Claude Code Stop hook: refuse to end a turn while the repo has uncommitted
# changes, and have Claude tag its commit with the session ID so the commit
# leads back to the conversation (`claude --resume <id>`).
# Nags once per stop (stop_hook_active), so changes that belong to another
# session can't trap Claude in a loop.
input=$(cat)
[ "$(jq -r .stop_hook_active <<<"$input")" = "true" ] && exit 0
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || exit 0
[ -z "$(git status --porcelain)" ] && exit 0
sid=$(jq -r .session_id <<<"$input")
jq -n --arg sid "$sid" '{
  decision: "block",
  reason: ("The repo has uncommitted changes. Commit ONLY the files you changed in this session (never `git add -A`), with a descriptive message ending in the trailer `Claude-Session: " + $sid + "`. If some changes are not yours, leave them alone and say which ones.")
}'
