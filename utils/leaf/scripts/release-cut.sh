#!/usr/bin/env sh
set -eu

usage() {
    echo "Usage: $0 <major|minor|patch> [branch]" >&2
    echo "Example: $0 patch main" >&2
    exit 1
}

BUMPTYPE="${1:-}"
BRANCH="${2:-main}"

[ -n "$BUMPTYPE" ] || usage

case "$BUMPTYPE" in
    major | minor | patch)
        ;;
    *)
        echo "Invalid bump type: $BUMPTYPE" >&2
        usage
        ;;
esac

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || {
    echo "Not inside a git repository" >&2
    exit 1
}

if ! git diff --quiet; then
    echo "Working tree has unstaged changes" >&2
    git status --short >&2
    exit 1
fi

if ! git diff --cached --quiet; then
    echo "Working tree has staged but uncommitted changes" >&2
    git status --short >&2
    exit 1
fi

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

git fetch origin "$BRANCH"
git fetch --tags origin

if ! git merge-base --is-ancestor "refs/remotes/origin/$BRANCH" HEAD; then
    echo "Current HEAD is behind or diverged from origin/$BRANCH" >&2
    exit 1
fi

CURRENT_VERSION="$(
    awk '
        /^\[package\]$/ { in_package = 1; next }
        /^\[/ && $0 != "[package]" { in_package = 0 }
        in_package && /^version = "/ {
            gsub(/^version = "/, "", $0)
            gsub(/"$/, "", $0)
            print
            exit
        }
    ' Cargo.toml
)"

case "$CURRENT_VERSION" in
    '' | *[!0-9.]* | *.*.*.* | .* | *.)
        echo "Invalid current version in Cargo.toml: $CURRENT_VERSION" >&2
        exit 1
        ;;
esac

IFS=.
set -- $CURRENT_VERSION
IFS=' '

CURRENT_MAJOR="${1:-}"
CURRENT_MINOR="${2:-}"
CURRENT_PATCH="${3:-}"

[ -n "$CURRENT_MAJOR" ] || {
    echo "Invalid current version in Cargo.toml: $CURRENT_VERSION" >&2
    exit 1
}
[ -n "$CURRENT_MINOR" ] || {
    echo "Invalid current version in Cargo.toml: $CURRENT_VERSION" >&2
    exit 1
}
[ -n "$CURRENT_PATCH" ] || {
    echo "Invalid current version in Cargo.toml: $CURRENT_VERSION" >&2
    exit 1
}

case "$CURRENT_MAJOR:$CURRENT_MINOR:$CURRENT_PATCH" in
    *[!0-9:]*)
        echo "Invalid current version in Cargo.toml: $CURRENT_VERSION" >&2
        exit 1
        ;;
esac

case "$BUMPTYPE" in
    major)
        NEXT_MAJOR=$((CURRENT_MAJOR + 1))
        NEXT_MINOR=0
        NEXT_PATCH=0
        ;;
    minor)
        NEXT_MAJOR=$CURRENT_MAJOR
        NEXT_MINOR=$((CURRENT_MINOR + 1))
        NEXT_PATCH=0
        ;;
    patch)
        NEXT_MAJOR=$CURRENT_MAJOR
        NEXT_MINOR=$CURRENT_MINOR
        NEXT_PATCH=$((CURRENT_PATCH + 1))
        ;;
esac

VERSION="$NEXT_MAJOR.$NEXT_MINOR.$NEXT_PATCH"
TAG="$VERSION"

if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null 2>&1; then
    echo "Tag already exists: $TAG" >&2
    exit 1
fi

if git ls-remote --tags --exit-code origin "refs/tags/$TAG" >/dev/null 2>&1; then
    echo "Remote tag already exists on origin: $TAG" >&2
    exit 1
fi

awk -v version="$VERSION" '
    BEGIN {
        in_package = 0
        replaced = 0
    }
    /^\[package\]$/ {
        in_package = 1
        print
        next
    }
    /^\[/ && $0 != "[package]" {
        in_package = 0
    }
    in_package && !replaced && /^version = "/ {
        print "version = \"" version "\""
        replaced = 1
        next
    }
    {
        print
    }
    END {
        if (!replaced) {
            exit 1
        }
    }
' Cargo.toml > "$tmp_file"
mv "$tmp_file" Cargo.toml

if git diff --quiet -- Cargo.toml; then
    echo "Cargo.toml already set to version $VERSION" >&2
    exit 1
fi

cargo update --workspace >/dev/null

PREV_TAG="$(
    git tag --sort=-v:refname \
        | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' \
        | head -n 1
)"
[ -n "$PREV_TAG" ] || {
    echo "No previous semver tag found" >&2
    exit 1
}

./scripts/changelog-update.sh "$PREV_TAG" "$TAG"

git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "$VERSION"
git tag -m "$TAG" "$TAG"
git push origin "HEAD:$BRANCH"
git push origin "$TAG"
