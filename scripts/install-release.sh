#!/usr/bin/env bash
# Install bridle from a GitHub release, with no clone and no Rust toolchain:
#   install-release.sh [tag]
# Downloads bridle-<tag>-<target>.tar.gz and SHA256SUMS (the names scripts/package-release.sh
# makes and crates/bridle-daemon/src/release.rs expects), refuses on a checksum mismatch, and
# puts `bridle` in ~/.local/bin. Default tag is the newest release.
# Env: BRIDLE_RELEASE_REPO (owner/name, default jonathanbranam/bridle),
#      BRIDLE_INSTALL_DIR (default ~/.local/bin),
#      BRIDLE_RELEASE_BASE (download base URL; for tests, a file:// dir holding the assets).
set -euo pipefail

repo=${BRIDLE_RELEASE_REPO:-jonathanbranam/bridle}
dest=${BRIDLE_INSTALL_DIR:-$HOME/.local/bin}
tag=${1:-}

# Same triples as release::target_for.
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64 | Darwin-aarch64) target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  *) echo "no bridle release for $(uname -s) $(uname -m)" >&2; exit 1 ;;
esac

if [ -z "$tag" ]; then
  # /releases/latest redirects to /releases/tag/<tag>; no API call, so no rate limit.
  tag=$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest")
  tag=${tag##*/}
fi
[ -n "$tag" ] || { echo "could not find the newest release of $repo" >&2; exit 1; }

base=${BRIDLE_RELEASE_BASE:-https://github.com/$repo/releases/download/$tag}
name=bridle-$tag-$target.tar.gz

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "installing bridle $tag ($target) from $repo"
curl -fsSL -o "$work/$name" "$base/$name"
curl -fsSL -o "$work/SHA256SUMS" "$base/SHA256SUMS"

want=$(awk -v n="$name" '$2 == n || $2 == "*" n { print $1 }' "$work/SHA256SUMS")
[ -n "$want" ] || { echo "SHA256SUMS doesn't list $name" >&2; exit 1; }
if command -v sha256sum >/dev/null 2>&1; then
  got=$(sha256sum "$work/$name" | awk '{ print $1 }')
else
  got=$(shasum -a 256 "$work/$name" | awk '{ print $1 }')
fi
if [ "$got" != "$want" ]; then
  echo "checksum mismatch for $name: expected $want, got $got; nothing installed" >&2
  exit 1
fi

mkdir "$work/x"
tar -xzf "$work/$name" -C "$work/x"
[ -f "$work/x/bridle" ] || { echo "the tarball has no bridle binary" >&2; exit 1; }

mkdir -p "$dest"
# Staged beside the target and renamed, so the path never holds a partial binary.
cp "$work/x/bridle" "$dest/bridle.new"
chmod 755 "$dest/bridle.new"
mv -f "$dest/bridle.new" "$dest/bridle"
echo "installed $dest/bridle"

case ":$PATH:" in
  *":$dest:"*) ;;
  *) echo "note: $dest is not on your PATH; add it (export PATH=\"$dest:\$PATH\" in your shell profile)" ;;
esac

cat <<NEXT
next: set [daemon] self_upgrade = "release" in ~/.bridle/config.toml so the daemon keeps itself
current, then follow docs/context/add-a-machine.md (bridle doctor checks the result).
NEXT
