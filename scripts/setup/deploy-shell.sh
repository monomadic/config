#!/bin/bash
# A minimal, standalone shell deployer. Run from the checkout with:
#   /bin/bash scripts/setup/deploy-shell.sh --check
#   /bin/bash scripts/setup/deploy-shell.sh
#
# Rationale: the mapping list below is the deployment definition. No Dotter,
# manifest parser, cache, network, or Homebrew is needed. The checkout root is
# resolved from this script, so the repository can live anywhere.
#
# link_file creates one absolute symlink. link_tree recursively links files,
# including hidden files, while leaving destination directories real: installed
# binaries and app-owned files can coexist with our links. Source directory
# symlinks are linked as single entries rather than traversed. Empty source
# directories do not create destination directories.
#
# Before making changes, check syntax, the Bash 3.2 baseline, required utilities,
# and every mapping. Run ShellCheck too when installed; its absence is allowed
# on a fresh machine. App version requirements must be added when their actual
# minimums are established; this example only checks deployment prerequisites.
#
# Correct links are left alone. Existing files and different links block the
# whole run. This first version deliberately has no backups, forced replacement,
# stale-link cleanup, or machine profiles. Relative links to the same source are
# reported as conflicts because link targets are compared literally.
#
# This is preflight, not a transaction: an unexpected apply-time failure can
# leave some links created; rerunning is safe. Do not run concurrent deploys or
# change destinations during deployment. Git pulls still change already-linked
# config immediately. --check performs the complete preflight without applying.
#
# These three mappings are an example, not a full migration of the Dotter setup.

set -euo pipefail
shopt -s nullglob dotglob

ROOT="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
phase=check
errors=0

case "${1:-}" in
    '') check_only=0 ;;
    --check) check_only=1 ;;
    *) printf 'Usage: %s [--check]\n' "$0" >&2; exit 2 ;;
esac
[[ $# -le 1 ]] || { printf 'Too many arguments.\n' >&2; exit 2; }

deploy() {
    link_file config/zsh/zshrc.zsh "$HOME/.zshrc"
    link_file config/kitty/kitty.conf "$HOME/.config/kitty/kitty.conf"
    link_tree bin "$HOME/.local/bin"
}

fail() {
    printf 'FAIL  %s\n' "$*" >&2
    errors=$((errors + 1))
}

link_file() {
    local source="$ROOT/$1" target="$2" parent

    if [[ "$phase" == check ]]; then
        if [[ ! -e "$source" ]]; then
            fail "Missing or broken source: $source"
            return
        fi
        if [[ -L "$target" ]]; then
            if [[ "$(readlink "$target")" != "$source" ]]; then
                fail "Different symlink already exists: $target"
            fi
        elif [[ -e "$target" ]]; then
            fail "File or directory already exists: $target"
        fi

        parent="$(dirname -- "$target")"
        while [[ ! -e "$parent" && ! -L "$parent" ]]; do
            parent="$(dirname -- "$parent")"
        done
        if [[ ! -d "$parent" || ! -w "$parent" || ! -x "$parent" ]]; then
            fail "Cannot create links beneath: $parent"
        fi
        return
    fi

    if [[ -L "$target" && "$(readlink "$target")" == "$source" ]]; then
        printf 'OK    %s\n' "$target"
        return
    fi
    mkdir -p -- "$(dirname -- "$target")"
    ln -sn -- "$source" "$target"
    printf 'LINK  %s\n' "$target"
}

link_tree() {
    local source="$1" target="$2" entry name
    if [[ ! -d "$ROOT/$source" || ! -r "$ROOT/$source" || ! -x "$ROOT/$source" ]]; then
        fail "Missing or unreadable source directory: $ROOT/$source"
        return
    fi
    for entry in "$ROOT/$source"/*; do
        name="${entry##*/}"
        if [[ -d "$entry" && ! -L "$entry" ]]; then
            link_tree "$source/$name" "$target/$name"
        else
            link_file "$source/$name" "$target/$name"
        fi
    done
}

preflight() {
    local command
    /bin/bash -n "$ROOT/scripts/setup/deploy-shell.sh"
    if (( BASH_VERSINFO[0] < 3 ||
          (BASH_VERSINFO[0] == 3 && BASH_VERSINFO[1] < 2) )); then
        fail 'Bash 3.2 or newer is required'
    fi
    for command in dirname mkdir ln readlink; do
        command -v "$command" >/dev/null 2>&1 || fail "Missing command: $command"
    done
    (( errors == 0 )) || exit 1
    if command -v shellcheck >/dev/null 2>&1; then
        shellcheck "$ROOT/scripts/setup/deploy-shell.sh"
    fi
    deploy
    if (( errors > 0 )); then
        printf '\nDeployment blocked: %s error(s). No changes made.\n' "$errors" >&2
        exit 1
    fi
    printf 'Preflight passed.\n'
}

preflight
(( check_only == 0 )) || exit 0
phase=apply
deploy
printf 'Deployment complete.\n'
