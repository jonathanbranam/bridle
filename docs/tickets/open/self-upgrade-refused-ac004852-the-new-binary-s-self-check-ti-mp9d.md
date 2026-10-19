---
id: br-mp9d
title: "Self-upgrade refused ac004852: the new binary's self-check timed out (3x60s); staged ad-hoc-signed binary hangs at exec"
kind: bug
created: 2026-10-19
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-p29s]
---

`bridle daemon restart --upgrade --project bridle` built ac004852e, then refused: "the new binary's self-check timed out (3 attempts of 60 s)". The daemon stayed on its old build. Found by the orchestrator (dalek), 2026-10-19 00:26Z.

Evidence:
- The staged binaries `.bridle/upgrade-root/bin/bridle` and `.bridle/upgrade-target/release/bridle` (Signature=adhoc, flags adhoc,linker-signed, Identifier `bridle-51496b437868e49b`) hang before printing anything, even for `--version`; `codesign -v` says valid; `sample` shows only the main thread.
- `~/.cargo/bin/bridle` (re-signed: Identifier `bridle`, no linker-signed flag) runs `--version` instantly and `serve --check` exits 0 in 1 s.
- Not told apart: the check ran a linker-signed copy that hangs at exec, or the first launch of a fresh binary stalls (macOS first-run assessment) and later launches are fine.

Likely area: crates/bridle-daemon/src/upgrade.rs (`check_built`, `exe_path()`), and the signing flow in CLAUDE.md (Build configuration; `signing::install`, `bridle sign binary`).

## The ask

Find which binary the self-check runs and why it hangs at exec; make the self-upgrade survive it: check the binary that will actually be exec'd, after any re-signing, or give a first-run assessment the time it needs. Verify: `restart --upgrade` on this Mac restarts into the new build.

Model: Sonnet. Investigate first and record the cause on this thread (a design doc line in docs/design/agent-host/ if the upgrade flow changes), then fix. Acceptance: just check; a unit test for whatever is testable without macOS (which binary is checked); the macOS behaviour verified by hand and reported on the thread. Out of scope: the "Verifying" popup (br-p29s) unless it is the same cause. Workaround meanwhile: plain `bridle daemon restart` after checking the installed binary by hand.
