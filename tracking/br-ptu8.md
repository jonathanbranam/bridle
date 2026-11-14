+++
id = "br-ptu8"
title = "Firewall prompts still appear on every self-upgrade despite the stable signing identity: find the break in the signing chain"
kind = "bug"
state = "open"
created_at = "2026-11-14T12:57:20.899Z"
updated_at = "2026-11-14T12:58:07.973166Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "ptu8"
+++

docs/tickets/open/firewall-prompts-still-appear-on-every-self-upgrade-despite-ptu8.md

## Thread

### note · external:aide · 2026-11-14T12:57:28.050Z
readied by external:aide

### note · external:aide · 2026-11-14T12:57:28.160Z
Readied by aide: a bug with a clear ask (rule ready-without-asking); the human asked for the cause to be found (quote above).

### note · agent:pm-1 · 2026-11-14T12:58:07.973Z
pm-1 brief: model Sonnet. This is diagnose-then-fix, in that order, on one branch.
1. Diagnose first, on the ticket thread, before changing code: read docs/tickets/resolved/ for qy4y, 7bs2 and p88z and crates/bridle/src/signing.rs. List every code path that writes ~/.cargo/bin/bridle (signing::install, the daemon self-upgrade, just install, `bridle sign binary`) and say for each whether it signs a copy named `bridle` and renames it into place; the 12:41:25Z install the aide found must be explained (which installer ran, did the 12:50Z upgrade re-install). Check candidates 1 and 4 from the ask from code and logs (the worker can run codesign, stat, plutil on the dalek machine it runs on).
2. Fix what the code shows (for example an install path that does not sign-and-rename, or a launchd restart before the rename). If the cause is the OS keying the firewall entry on file identity (candidate 3) or needs a dialog to reproduce, stop and post findings; the aide gets the human to run one upgrade at dalek, and the fix follows on this ticket or a slice.
3. Do not switch to localhost plus tailscale serve (human's go needed). Candidate 2 (reboot state) is a human-at-dalek check, not the worker's.
Acceptance: findings on the thread; any code fix has a test of the install ordering (signed before the restart, atomic rename); `just check` with BRIDLE_* env unset. Docs: the signing section of CLAUDE.md or docs/design where the chain is described, if behaviour changes.
