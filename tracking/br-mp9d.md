+++
id = "br-mp9d"
title = "Self-upgrade refused ac004852: the new binary's self-check timed out (3x60s); staged ad-hoc-signed binary hangs at exec"
kind = "bug"
state = "planned"
created_at = "2026-10-19T00:40:28.426Z"
updated_at = "2026-10-19T01:26:16.947598Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "mp9d"
+++

docs/tickets/open/self-upgrade-refused-ac004852-the-new-binary-s-self-check-ti-mp9d.md

submitted by external:orchestrator

Found by orchestrator (dalek), 2026-10-19 00:26Z.

`bridle daemon restart --upgrade --project bridle` built ac004852e, then: `[upgrade_failed] upgrade: built ac004852e but refused to restart into it: the new binary's self-check timed out (3 attempts of 60 s)`. The daemon stayed on its old build (pre br-fyww).

What I found (00:35-00:45Z):
- `.bridle/upgrade-root/bin/bridle` and `.bridle/upgrade-target/release/bridle` (20:22 local, `Signature=adhoc`, flags `adhoc,linker-signed`, Identifier `bridle-51496b437868e49b`) hang before printing anything, even for `--version` (killed at 20s; `serve --check` killed at 150s; `RUST_LOG=debug` prints nothing; `sample` shows only the main thread in an unknown frame). `codesign -v` says valid.
- `~/.cargo/bin/bridle` (20:23 local, re-signed: Identifier `bridle`, flags none) runs `--version` instantly and `--project bridle serve --check --repo /Volumes/Data/work/bridle/bridle --workspace /Volumes/Data/work/bridle` exits 0 in 1 s.
- So either the check ran a copy that hangs at exec (the linker-signed staged binary), or the first launch of the freshly installed binary stalled (macOS first-run assessment) and later launches are fine. Not told apart.

Cause: unknown; macOS code-signing / exec assessment of the new binary is the likely area (crates/bridle-daemon/src/upgrade.rs check_built; exe_path()).

## The ask

Find which binary the self-check runs and why it hangs at exec, and make the self-upgrade survive it (check the binary that will actually be exec'd, after any re-signing; or give a first-run assessment the time it needs). Verify: a `restart --upgrade` on this Mac restarts into the new build.

Workaround used: plain `bridle daemon restart --project bridle` after checking the installed binary by hand.

## Thread

### note · external:orchestrator · 2026-10-19T00:40:28.443Z
submitted by external:orchestrator

### note · external:aide · 2026-10-19T00:53:21.927Z
The human, 2026-10-18 ~8:45 PM ET (to aide), verbatim, after checking dalek's screen: "there is no open prompte, but the rapid dialog shows up and hides and eats a few keystrokes when it does; it happens in bursts when something is building / testing i assume". So no dialog is waiting on an answer. Aide also saw on dalek ~8:35 PM ET: syspolicyd at 40-250% CPU, 32 test binaries stuck ~45 s each at their first --list exec, and bridle/integration/target at 90 GB (~788K files in debug/deps). The hang looks like the syspolicyd scan queue (p29s/j4bk), not a dialog.
