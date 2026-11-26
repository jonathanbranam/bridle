#!/usr/bin/env bash
# Cut a bridle release: scripts/release.sh [patch|minor|major|X.Y.Z] [-y] [-n]
#
# Does what a release has always been by hand: bump [workspace.package] version in Cargo.toml,
# refresh Cargo.lock, put a dated heading under "## Unreleased" in CHANGELOG.md, move ui.pin to
# bridle-ui main (refusing unless its CI is green and its API version is the gateway's), commit only
# those four files (the tree often holds others' uncommitted ticket edits), tag vX.Y.Z and push main
# and the tag. The tag push starts .github/workflows/release.yml, which builds and publishes.
#
# Run it from your own terminal on the owner's clone: the pre-push hook refuses elsewhere, and
# agents can't commit here. -y skips the confirmation; -n (dry run) checks and prints the plan only.
set -euo pipefail

usage() { sed -n '2,11p' "$0" | sed 's/^# \{0,1\}//'; exit "${1:-0}"; }

bump=patch yes=0 dry=0
for a in "$@"; do
  case "$a" in
    -y) yes=1 ;;
    -n) dry=1 ;;
    -h|--help) usage ;;
    patch|minor|major) bump=$a ;;
    [0-9]*.[0-9]*.[0-9]*) bump=$a ;;
    *) echo "unknown argument: $a" >&2; usage 2 ;;
  esac
done

die() { echo "release: $*" >&2; exit 1; }

cd "$(git rev-parse --show-toplevel)"
[ -f Cargo.toml ] && [ -f CHANGELOG.md ] || die "not the bridle repo"

branch=$(git symbolic-ref --short HEAD)
[ "$branch" = main ] || die "on $branch, not main"
git diff --quiet HEAD -- Cargo.toml Cargo.lock CHANGELOG.md ui.pin \
  || die "Cargo.toml, Cargo.lock, CHANGELOG.md or ui.pin has uncommitted changes"
git fetch -q origin main --tags
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] \
  || die "main is not origin/main (pull or push first)"

cur=$(awk '/^\[workspace.package\]/{s=1;next} /^\[/{s=0} s && /^version *=/{gsub(/[" ]/,"",$0); sub(/version=/,""); print; exit}' Cargo.toml)
[ -n "$cur" ] || die "no [workspace.package] version in Cargo.toml"
IFS=. read -r ma mi pa <<<"$cur"
case "$bump" in
  patch) new="$ma.$mi.$((pa + 1))" ;;
  minor) new="$ma.$((mi + 1)).0" ;;
  major) new="$((ma + 1)).0.0" ;;
  *) new=$bump ;;
esac
tag="v$new"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && die "$tag already exists"

# Something to release: entries between "## Unreleased" and the next heading.
entries=$(awk '/^## Unreleased/{s=1;next} /^## /{s=0} s && /^- /' CHANGELOG.md | wc -l | tr -d ' ')
[ "$entries" -gt 0 ] || die "CHANGELOG.md has nothing under ## Unreleased"

# CI on this commit: the release is cut from a green main.
ci="unknown (no gh)"
if command -v gh >/dev/null; then
  ci=$(gh run list --branch main --commit "$(git rev-parse HEAD)" --json conclusion,status \
    -q 'if length == 0 then "no run" elif any(.[]; .status != "completed") then "running"
        elif all(.[]; .conclusion == "success") then "success" else "failed" end' 2>/dev/null) \
    || ci="unknown (gh failed)"
fi

# The release ships the UI at ui.pin, so it moves to bridle-ui main every time (vdvw); a main that
# isn't known to be green stops the release rather than shipping an older UI.
ui=$(scripts/check-ui-pin.sh --latest) || die "cannot find bridle-ui main"
command -v gh >/dev/null || die "gh is needed to check bridle-ui's CI"
ui_ci=$(gh run list -R jonathanbranam/bridle-ui --branch main --commit "$ui" --json conclusion,status \
  -q 'if length == 0 then "no run" elif any(.[]; .status != "completed") then "running"
      elif all(.[]; .conclusion == "success") then "success" else "failed" end') \
  || die "cannot read bridle-ui's CI"
[ "$ui_ci" = success ] || die "bridle-ui main $ui: CI is '$ui_ci', not success"
# release.yml checks the built UI too, but only after the tag is pushed.
api_want=$(sed -n 's/^pub const API_VERSION: u32 = \([0-9]*\);.*/\1/p' crates/bridle-gateway/src/lib.rs)
api_got=$(gh api "repos/jonathanbranam/bridle-ui/contents/src/api/version.ts?ref=$ui" \
  -H 'Accept: application/vnd.github.raw' | sed -n 's/^export const API_VERSION = \([0-9]*\);.*/\1/p') \
  || die "cannot read bridle-ui's API version"
[ -n "$api_want" ] && [ "$api_got" = "$api_want" ] \
  || die "bridle-ui main $ui: API version '$api_got' != gateway API_VERSION '$api_want'"

echo "release $cur -> $new ($tag) from $(git log -1 --format='%h %s')"
echo "  $entries changelog entries; CI on this commit: $ci"
echo "  ui.pin: $(cut -c1-7 ui.pin) -> ${ui:0:7} (bridle-ui main, CI green, API version $api_got)"
case "$ci" in
  success) ;;
  failed) die "CI failed on this commit" ;;
  *) echo "  warning: CI is not green yet" ;;
esac
[ "$dry" = 1 ] && { echo "dry run: nothing changed"; exit 0; }
if [ "$yes" = 0 ]; then
  read -r -p "cut $tag and push? [y/N] " ok
  [ "$ok" = y ] || [ "$ok" = Y ] || die "stopped"
fi

tmp=$(mktemp)
awk -v v="$new" '/^\[workspace.package\]/{s=1} /^\[/ && !/^\[workspace.package\]/{s=0}
  s && /^version *=/ && !done {print "version = \"" v "\""; done=1; next} {print}' Cargo.toml >"$tmp"
cat "$tmp" >Cargo.toml
awk -v h="## [$new] - $(date +%Y-%m-%d)" '{print} /^## Unreleased/ && !done {print ""; print h; done=1}' \
  CHANGELOG.md >"$tmp"
cat "$tmp" >CHANGELOG.md
rm -f "$tmp"
echo "$ui" >ui.pin
# Rewrites the workspace members' versions in Cargo.lock; no build, no network.
cargo metadata --format-version 1 --offline >/dev/null

git commit -q -m "Release $new" -- Cargo.toml Cargo.lock CHANGELOG.md ui.pin
git tag -a "$tag" -m "Release $new"
git push -q origin main "$tag"
echo "pushed $tag: release.yml builds it; daemons on release mode pick it up from there"
