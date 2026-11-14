+++
id = "br-ptu8"
title = "Firewall prompts still appear on every self-upgrade despite the stable signing identity: find the break in the signing chain"
kind = "bug"
state = "open"
created_at = "2026-11-14T12:57:20.899Z"
updated_at = "2026-11-14T13:13:00.617954Z"
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

### note · external:aide · 2026-11-14T13:12:36.799Z
The human, 2026-11-14 (to aide), verbatim: "hmm... yeah let's verify before that; it sounds risky. Can we do some more internet research on the os x behavior? It should be clearly documented. I don't understand why we are guessing here. Do i need to find and download some docs from apples developer resource center?"

Change to the brief (aide, for the human):
1. FIRST, research, by a worker with the `researcher` role (web tools): what Apple documents about how the macOS application firewall (socketfilterfw / ALF) identifies an allowed app and when it re-prompts: code signature vs designated requirement vs cdhash vs path/inode; self-signed certificates; behaviour after a binary is replaced or updated in place; after reboot. Cite sources (Apple support and developer docs, Platform Security Guide, man socketfilterfw, Apple DTS forum answers by Apple staff such as Quinn), and say plainly where Apple documents nothing and what is only folklore.
2. Then verify the cause on dalek with evidence, as the brief already says.
3. NO change to how bridle installs (no in-place overwrite, no tailscale serve) until the cause is verified AND the human has approved the fix. The human called the in-place overwrite "risky". Post findings and the proposed fix on this thread; the aide brings them to the human.

### note · agent:pm-1 · 2026-11-14T13:13:00.617Z
pm-1 REVISED brief (supersedes my 12:58Z brief; follows the human's change above and the orchestrator's note). This ticket is now research and verification ONLY. No code change, no install-path change.

Worker: role `researcher` (web tools); model Sonnet. Ask manager-2 to spawn it with that role; if it cannot, say so on this thread.

1. Research. What Apple documents about how the macOS application firewall (socketfilterfw / ALF) identifies an allowed app and when it re-prompts: code signature vs designated requirement vs cdhash vs path or inode; self-signed certificates; a binary replaced by rename-into-place vs overwritten in place; behaviour after reboot or sleep. Sources: Apple support and developer docs, the Platform Security Guide, `man socketfilterfw`, Apple DTS forum answers by Apple staff (Quinn). Include the c2 wiki per rule c2-wiki-in-searches if it has a relevant page (say if unreachable). Say plainly where Apple documents nothing and what is only folklore. Read first: resolved tickets qy4y, 7bs2 and p88z, and the orchestrator's evidence note above (signing::install runs as soon as the build finishes, before the drain; the dialog appeared within a minute of that install, before the 12:50Z restart; the daemon log shows a sleep clock jump at 12:31Z).
2. Verify on dalek with evidence, using only read-only commands (codesign, stat, plutil -p on alf.plist, log show for socketfilterfw, listapps). Any experiment that raises a dialog goes through the aide, since the human is at dalek today; the worker proposes the exact steps and waits.
3. Post findings on this thread: the cause (proven, or ranked candidates with what each test showed), citations, and a PROPOSED fix with its risks (for example in-place overwrite, which the human called risky). The aide takes it to the human. A fix is a new ticket after the human approves; this ticket resolves when the cause is established or the research is shown inconclusive.

Out of scope: any change to signing.rs, upgrade.rs, install or launchd handling; tailscale serve. Acceptance: findings on the thread with citations, nothing merged.
