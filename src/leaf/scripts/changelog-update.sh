#!/usr/bin/env sh
set -eu

usage() {
    echo "Usage: $0 <previous_tag> <new_tag>" >&2
    echo "Example: $0 1.9.1 1.10.0" >&2
    exit 1
}

PREV_TAG="${1:-}"
NEW_TAG="${2:-}"

[ -n "$PREV_TAG" ] || usage
[ -n "$NEW_TAG" ] || usage

REPO_URL="https://github.com/${GITHUB_REPOSITORY:-RivoLink/leaf}"
DATE_UTC="$(date -u +%Y-%m-%d)"
SENTINEL="<!-- next-version -->"

grep -q "^${SENTINEL}$" CHANGELOG.md || {
    echo "Missing sentinel '${SENTINEL}' in CHANGELOG.md" >&2
    exit 1
}

# Parse each merge commit: subject carries "#N", body's first
# non-empty line is the PR title. One output line per PR.
PR_LINES="$(
    git log --first-parent --merges "$PREV_TAG..HEAD" \
        --pretty='format:%s%x09%b%x00' \
    | awk -v url="$REPO_URL" 'BEGIN{RS="\0"; FS="\t"} NF {
        if (match($1, /#[0-9]+/)) {
            pr_full = substr($1, RSTART, RLENGTH)
            pr_num  = substr(pr_full, 2)
        } else { pr_full = ""; pr_num = "" }
        n = split($2, b, "\n"); title = ""
        for (i = 1; i <= n; i++) if (b[i] != "") { title = b[i]; break }
        if (title != "") {
            if (pr_full != "")
                print title " ([" pr_full "](" url "/pull/" pr_num "))"
            else
                print title
        }
    }'
)"

[ -n "$PR_LINES" ] || {
    echo "No merged PR found between $PREV_TAG and $NEW_TAG" >&2
    exit 1
}

DOCS_FILE="$(mktemp)"
ADDED_FILE="$(mktemp)"
FIXED_FILE="$(mktemp)"
CHANGED_FILE="$(mktemp)"
NEW_SECTION_FILE="$(mktemp)"
TMP_CHANGELOG="$(mktemp)"
trap 'rm -f "$DOCS_FILE" "$ADDED_FILE" "$FIXED_FILE" "$CHANGED_FILE" "$NEW_SECTION_FILE" "$TMP_CHANGELOG"' EXIT

printf '%s\n' "$PR_LINES" | while IFS= read -r line; do
    [ -z "$line" ] && continue
    case "$line" in
        docs:*|docs\ *) target="$DOCS_FILE" ;;
        feat:*|feat\ *) target="$ADDED_FILE" ;;
        fix:*|fix\ *)   target="$FIXED_FILE" ;;
        *)              target="$CHANGED_FILE" ;;
    esac
    stripped="$(printf '%s' "$line" | sed -E 's/^[a-z]+: *//')"
    [ -n "$stripped" ] || continue
    printf -- '- %s\n' "$stripped" >> "$target"
done

{
    printf '## [[%s](%s/releases/tag/%s)] - %s\n\n' \
        "$NEW_TAG" "$REPO_URL" "$NEW_TAG" "$DATE_UTC"
    for pair in "Docs:$DOCS_FILE" "Added:$ADDED_FILE" \
                "Fixed:$FIXED_FILE" "Changed:$CHANGED_FILE"; do
        name="${pair%%:*}"
        file="${pair#*:}"
        if [ -s "$file" ]; then
            printf '### %s\n\n' "$name"
            cat "$file"
            printf '\n'
        fi
    done
} > "$NEW_SECTION_FILE"

awk -v section_file="$NEW_SECTION_FILE" '
    /^<!-- next-version -->$/ {
        print
        print ""
        while ((getline line < section_file) > 0) print line
        # Drop the line if blank (avoid double blank); preserve it
        # otherwise so content is not lost on a manually edited file.
        if ((getline extra) > 0 && extra != "") print extra
        next
    }
    { print }
' CHANGELOG.md > "$TMP_CHANGELOG"

# mktemp defaults to 0600
# preserve the original CHANGELOG.md mode.
chmod --reference=CHANGELOG.md "$TMP_CHANGELOG"
mv "$TMP_CHANGELOG" CHANGELOG.md
