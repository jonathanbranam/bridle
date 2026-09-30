#!/usr/bin/env bash
# Package a built bridle binary: package-release.sh <tag> <target> <binary> <outdir>
# Writes <outdir>/bridle-<tag>-<target>.tar.gz containing just `bridle`.
set -euo pipefail
tag=$1 target=$2 bin=$3 out=$4
mkdir -p "$out"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp "$bin" "$stage/bridle"
tar -C "$stage" -czf "$out/bridle-$tag-$target.tar.gz" bridle
