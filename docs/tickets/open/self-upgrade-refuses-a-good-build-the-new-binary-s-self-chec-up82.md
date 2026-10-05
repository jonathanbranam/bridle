---
id: up82
title: "Self-upgrade refuses a good build: the new binary's self-check timed out (60 s) on the Intel Mac under load"
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

Stop a good build from failing the upgrade self-check just because it is the binary's first run on a loaded machine.

## What happened

2026-10-05 03:46-03:51Z: the automatic upgrade built 0d69d617 (CI green) and then refused to restart into it: "the new binary's self-check timed out" (upgrade.rs `check_built`, a 60 s cap on `bridle serve --check`). Two workers' `just check` runs were going at the time. Ten minutes later the orchestrator ran the same self-check by hand on the same installed binary: rc 0 in under a second.

The likely cause, unverified: the first exec of a freshly linked ad-hoc-signed binary on this Intel Mac waits on macOS's policy check (syspolicyd; see sign-binaries-on-intel-macs-cs7x), which is slow under load. Later runs are fast. `security find-identity` shows no "bridle local signing" identity on dalek, so the p88z re-sign doesn't apply and the binary stays ad-hoc.

Side effect: `cargo install` had already replaced `~/.cargo/bin/bridle`, so the CLI runs 0d69d617 while the daemon is still c548182b.

## Options

- Retry the self-check once (or twice) before failing; the second run is the cheap, direct test of "first launch is slow".
- Or warm the binary first (e.g. `bridle --version`) with its own generous timeout, then run the check with the 60 s cap.
- Confirm or rule out the syspolicyd guess (`log show --predicate 'process == "syspolicyd"'` around 03:50Z).
