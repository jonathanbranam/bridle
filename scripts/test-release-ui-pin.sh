#!/usr/bin/env bash
# Offline test of check-ui-pin.sh and release.sh's ui.pin move (vdvw): a fixture bridle-ui, a
# fixture bridle repo with a local bare origin (nothing leaves the temp dir) and a stub gh.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
t=$(mktemp -d)
trap 'rm -rf "$t"' EXIT
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@example.invalid
export GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@example.invalid
fail() { echo "FAIL: $*" >&2; exit 1; }

git init -q -b main "$t/ui"
git -C "$t/ui" commit -q --allow-empty -m one
old=$(git -C "$t/ui" rev-parse HEAD)
git -C "$t/ui" commit -q --allow-empty -m two
new=$(git -C "$t/ui" rev-parse HEAD)
export BRIDLE_UI_REPO="$t/ui"

echo "$old" >"$t/pin"
if "$here/check-ui-pin.sh" "$t/pin" >/dev/null 2>&1; then fail "a stale pin passed"; fi
echo "$new" >"$t/pin"
"$here/check-ui-pin.sh" "$t/pin" >/dev/null || fail "a current pin failed"

# gh stub: bridle-ui's run list answers $UI_CI and its version.ts $UI_API; bridle's CI is success.
mkdir "$t/bin"
cat >"$t/bin/gh" <<'STUB'
#!/usr/bin/env bash
case "$*" in
  *version.ts*) echo "export const API_VERSION = $UI_API;" ;;
  *bridle-ui*) echo "$UI_CI" ;;
  *) echo success ;;
esac
STUB
chmod +x "$t/bin/gh"

git init -q --bare -b main "$t/origin.git"
git init -q -b main "$t/br"
mkdir -p "$t/br/scripts" "$t/br/member/src" "$t/br/crates/bridle-gateway/src"
echo 'pub const API_VERSION: u32 = 2;' >"$t/br/crates/bridle-gateway/src/lib.rs"
cp "$here/release.sh" "$here/check-ui-pin.sh" "$t/br/scripts/"
printf '[workspace]\nmembers = ["member"]\nresolver = "2"\n\n[workspace.package]\nversion = "0.1.0"\n' >"$t/br/Cargo.toml"
printf '[package]\nname = "member"\nversion.workspace = true\n' >"$t/br/member/Cargo.toml"
touch "$t/br/member/src/lib.rs"
printf '# Changelog\n\n## Unreleased\n\n- something\n' >"$t/br/CHANGELOG.md"
echo "$old" >"$t/br/ui.pin"
(cd "$t/br" && cargo generate-lockfile --offline -q)
git -C "$t/br" add -A
git -C "$t/br" commit -q -m init
git -C "$t/br" remote add origin "$t/origin.git"
git -C "$t/br" push -q origin main

release() { (cd "$t/br" && PATH="$t/bin:$PATH" UI_CI=$1 UI_API=$2 scripts/release.sh -y >/dev/null 2>&1); }

if release failed 2; then fail "released with bridle-ui CI failed"; fi
if release success 1; then fail "released with a mismatched API version"; fi
[ "$(cat "$t/br/ui.pin")" = "$old" ] || fail "ui.pin moved by a refused release"
[ -z "$(git -C "$t/origin.git" tag)" ] || fail "a refused release pushed a tag"

release success 2 || fail "release with a green bridle-ui failed"
[ "$(git -C "$t/origin.git" show v0.1.1:ui.pin)" = "$new" ] || fail "the tag's ui.pin is not bridle-ui main"
echo "release ui.pin: ok"
