#!/usr/bin/env bash
# Package a built bridle binary: package-release.sh <tag> <target> <binary> <outdir> [<ui-dir>]
# Writes <outdir>/bridle-<tag>-<target>.tar.gz containing `bridle` and, when <ui-dir> is given,
# its contents as `ui/` (the built bridle-ui; the daemon and install-release.sh put it in
# ~/.bridle/ui/).
set -euo pipefail
tag=$1 target=$2 bin=$3 out=$4 ui=${5:-}
mkdir -p "$out"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp "$bin" "$stage/bridle"
members=(bridle)
if [ -n "$ui" ]; then
  [ -d "$ui" ] || { echo "no ui dir at $ui" >&2; exit 1; }
  cp -R "$ui" "$stage/ui"
  members+=(ui)
fi
tar -C "$stage" -czf "$out/bridle-$tag-$target.tar.gz" "${members[@]}"
