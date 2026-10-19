+++
id = "br-mp9d"
title = "Self-upgrade refused ac004852: the new binary's self-check timed out (3x60s); staged ad-hoc-signed binary hangs at exec"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T00:40:28.426Z"
updated_at = "2026-10-19T10:48:26.768470Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wmp9d",
]
branch = "bridle/wmp9d"
commit = "1247ee95946383b11ef1bb1c0226e781d0a0a8d2"
summary = """
Investigation (wmp9d). Which binary: check_built runs crate::exe_path(), the live installed path. run_build installs the built binary there via signing::install (re-signed, renamed into place) BEFORE the check, so the check already runs the binary launchd will exec. It does not run the staged upgrade-root/upgrade-target copies; those hangs in the evidence were manual runs of the staged copies. So "wrong binary checked" is ruled out by the code.
Cause, judged from code and events: load-driven, intermittent. The installed copy is a fresh file; its first exec waits on macOS's syspolicyd scan, which was at 40-250% CPU with 32 test binaries stuck ~45 s each at first exec and a 90 GB integration/target (br-mhfx, landed, prunes it). Under that queue one exec can exceed 60 s; the retry kills the child and restarts the wait, so 3x60 s can all time out. The 2026-10-19 01:51Z upgrade to f5479f0c succeeded (upgrade.built/draining events), consistent with load, not a deterministic linker-signed hang.
Fix: CHECK_TIMEOUT 60 s -> 180 s (3 attempts kept); the command is now real_check_command(exe, ws) with a unit test that it runs the given live binary. daemon.md line and CHANGELOG (Fixed) updated.
Verify by hand (not run here): on the Mac, under some build load, `bridle daemon restart --upgrade --project bridle` should restart into the new build; upgrade.* events show built then draining."""
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

### note · agent:wmp9d · 2026-10-19T10:47:53.549Z
Investigation (wmp9d). Which binary: check_built runs crate::exe_path(), the live installed path. run_build installs the built binary there via signing::install (re-signed, renamed into place) BEFORE the check, so the check already runs the binary launchd will exec. It does not run the staged upgrade-root/upgrade-target copies; those hangs in the evidence were manual runs of the staged copies. So "wrong binary checked" is ruled out by the code.
Cause, judged from code and events: load-driven, intermittent. The installed copy is a fresh file; its first exec waits on macOS's syspolicyd scan, which was at 40-250% CPU with 32 test binaries stuck ~45 s each at first exec and a 90 GB integration/target (br-mhfx, landed, prunes it). Under that queue one exec can exceed 60 s; the retry kills the child and restarts the wait, so 3x60 s can all time out. The 2026-10-19 01:51Z upgrade to f5479f0c succeeded (upgrade.built/draining events), consistent with load, not a deterministic linker-signed hang.
Fix: CHECK_TIMEOUT 60 s -> 180 s (3 attempts kept); the command is now real_check_command(exe, ws) with a unit test that it runs the given live binary. daemon.md line and CHANGELOG (Fixed) updated.
Verify by hand (not run here): on the Mac, under some build load, `bridle daemon restart --upgrade --project bridle` should restart into the new build; upgrade.* events show built then draining.

### note · agent:wmp9d · 2026-10-19T10:47:57.389Z
built: 91cf84df (branch bridle/wmp9d)

### note · agent:wmp9d · 2026-10-19T10:47:57.469Z
done: self-check timeout 60->180 s per run (code already checked the live re-signed binary; cause is load-driven syspolicyd first-exec queue, see thread); just check exit 0, 1888 passed; 91cf84df. Hand-verify restart --upgrade on the Mac (not run).

### note · agent:manager-2 · 2026-10-19T10:48:11.962Z
integrated: 1247ee95946383b11ef1bb1c0226e781d0a0a8d2 (branch bridle/wmp9d)

### note · agent:manager-2 · 2026-10-19T10:48:26.768Z
cleanup: removed agent wmp9d, branch bridle/wmp9d
