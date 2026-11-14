+++
id = "br-ptu8"
title = "Firewall prompts still appear on every self-upgrade despite the stable signing identity: find the break in the signing chain"
kind = "bug"
state = "open"
created_at = "2026-11-14T12:57:20.899Z"
updated_at = "2026-11-14T12:58:58.081340Z"
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

### note · external:orchestrator · 2026-11-14T12:58:58.081Z
orchestrator: the 12:41:25Z installer was the bridle daemon's own self-upgrade to 8352cc3, not my daemon restarts. Evidence:
- bridle/.bridle/upgrade-root/bin/bridle mtime 12:41:24Z.
- ~/.cargo/bin/bridle born 12:41:24Z (new inode 194061122), signed 12:41:25Z, Authority=bridle local signing, Identifier=bridle, CDHash ef3d0ee3.
- run_build() in crates/bridle-daemon/src/upgrade.rs calls signing::install (copy, codesign, rename over the live path) as soon as the build finishes. It does not wait for the drain. The bridle daemon then drained on worker nxkj and only restarted at 12:50:27Z.
- The gateway saw the changed binary at 12:41:34Z ('the bridle binary changed; restarting onto it'), re-exec'd onto the new inode, and at 12:42:34Z raised 'cannot connect to its own address 100.100.189.100:7878'. So the dialog appeared within a minute of the install.
- My restarts of bridle-ui, data-contracts and track-web (12:41:21-26Z) only logged the signature ('installed binary signature before restart'; log_signature only reads). They wrote nothing.
- Same sequence as before: a new inode at the same path, the same identifier, and the same certificate leaf 37e545f6. The firewall still asked again. That fits br-7bs2's finding that the decision is keyed by path plus something the rename changes (cdhash?), not the designated requirement alone.
- Side note: 'the clock jumped: the machine slept' at 12:31:08Z in the bridle daemon log. That is relevant to the human's reboot/timing suspicion.
