#!/usr/bin/env bash
# Fail unless ui.pin is bridle-ui main's head (vdvw): check-ui-pin.sh [<pin-file>]
#   check-ui-pin.sh --latest   prints bridle-ui main's head, the sha the pin should hold
# bridle-ui is public, so this needs the network but no token. BRIDLE_UI_REPO overrides the
# repository (tests).
set -euo pipefail
repo=${BRIDLE_UI_REPO:-https://github.com/jonathanbranam/bridle-ui.git}
die() { echo "check-ui-pin: $*" >&2; exit 1; }

latest=$(git ls-remote "$repo" refs/heads/main | cut -f1) || die "cannot read $repo"
[ -n "$latest" ] || die "no main branch in $repo"
if [ "${1:-}" = --latest ]; then
  echo "$latest"
  exit 0
fi

file=${1:-"$(dirname "$0")/../ui.pin"}
pin=$(tr -d '[:space:]' <"$file") || die "cannot read $file"
[ "$pin" = "$latest" ] \
  || die "stale ui.pin: it names '$pin' but bridle-ui main is $latest; cut the release with scripts/release.sh, which moves it"
echo "ui.pin is current: bridle-ui main $latest"
