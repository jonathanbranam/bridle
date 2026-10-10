#!/usr/bin/env bash
# Offline test of install-release.sh against a fixture release in a temp dir.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
t=$(mktemp -d)
trap 'rm -rf "$t"' EXIT

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64 | Darwin-aarch64) target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  *) echo "skip: no release target for this platform"; exit 0 ;;
esac

mkdir "$t/rel"
printf '#!/bin/sh\necho fixture\n' > "$t/bridle"
chmod 755 "$t/bridle"
"$here/package-release.sh" v9.9.9 "$target" "$t/bridle" "$t/rel"
name=bridle-v9.9.9-$target.tar.gz
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$t/rel" && sha256sum "$name" > SHA256SUMS)
else
  (cd "$t/rel" && shasum -a 256 "$name" > SHA256SUMS)
fi

export BRIDLE_RELEASE_BASE="file://$t/rel" BRIDLE_INSTALL_DIR="$t/bin"
"$here/install-release.sh" v9.9.9 > /dev/null
[ "$("$t/bin/bridle")" = fixture ] || { echo "FAIL: installed binary wrong"; exit 1; }

rm -rf "$t/bin"
echo "0000000000000000000000000000000000000000000000000000000000000000  $name" > "$t/rel/SHA256SUMS"
if "$here/install-release.sh" v9.9.9 > /dev/null 2>&1; then
  echo "FAIL: bad checksum was accepted"; exit 1
fi
[ ! -e "$t/bin/bridle" ] || { echo "FAIL: binary installed despite bad checksum"; exit 1; }
echo "ok"
