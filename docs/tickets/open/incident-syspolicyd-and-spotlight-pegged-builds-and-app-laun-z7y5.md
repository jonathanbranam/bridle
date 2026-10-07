---
id: z7y5
title: "Incident: syspolicyd and Spotlight pegged, builds and app launches stalled: each spawn clones a 564K-file integration target/"
kind: incident
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [b7cz, y455]
tasks: [br-z7y5]
---

## The ask

Reported by the human, 2026-10-06 ~7:25 PM ET, with a diagnosis they pasted in: Finder and other app launches hung, the Report dialog hung. Their task: "As orchestrator - it's your job to keep the machine and builds running. ... investigate, file an incident, and we can consider a postmortem as well."

## What was seen (23:29 UTC)

- syspolicyd at 290-334% CPU; mds and mds_stores at 25-65%, about 10 mdworkers; spindump busy. The load average was 12-19 on 16 cores, mostly processes waiting. Memory was fine (69% free).
- About 33 cargo/rustc processes at ~0% CPU in wt/y455-postmortem (clippy --workspace, cargo check, nextest run), some rustc jobs on tiny crates running for ~9 min. Each proc-macro .dylib it loads is new and ad-hoc signed, so syspolicyd assesses it first.
- The bridle daemon (pid 15091) running `cp -cR /Volumes/Data/work/bridle/integration/target /Volumes/Data/work/bridle/wt/send-fix/target` (pid 88006) for 14+ min, state U (waiting on the disk).

## Cause (found)

`integration/target/debug/deps` holds **564,137 files**; the clone's `target/debug/deps` has 9,283. `warm_target` (`crates/bridle-daemon/src/worktree.rs`, b7cz, br-14cd) clones the integration target/ into every new worktree with `cp -cR` while the spawn waits. The integration target/ is rebuilt after each land (`warm_build.rs`) and never pruned, so stale hashed artifacts pile up. APFS cloning makes the data free, but not the per-file work: half a million new inodes, FSEvents for each, Spotlight indexing them, and syspolicyd assessing each new executable or dylib as it's launched. The send-fix copy did ~51K files in 14 min, so a whole copy would take ~2.5 h.

There are 19 worktrees with a target/ under wt/, each a partial or full copy.

## Impact

- The machine: Finder, app launches and crash reporting stalled for the human.
- Builds: every build on the machine waits behind syspolicyd. Workers' `just check` runs time out (page-title-rule2's hung check, 2026-10-06, was likely this).
- Spawns: the spawn waits on the copy. The worker for br-x56y (critical: no agent can `bridle send`) can't start for hours.
- Likely linked to y455 (spawn calls hanging 120-300 s since about 2026-10-05 23:30 UTC; a stuck 'spawning' flag): a spawn blocked in `warm_target` holds the flag for as long as the copy runs. To be confirmed in the postmortem.

## Follow-ups (to decide; options, not a plan)

- Now: stop the copy (pid 88006) and remove the partial `wt/send-fix/target`; prune or delete `integration/target` (the next warm build rebuilds it).
- Code: cap or prune the warm source (cargo-sweep style, or clean when it passes a file count), and don't hold the spawn on the copy (or skip warming when the source is huge).
- Machine (the human's): exempt iTerm in Privacy & Security > Developer Tools, then restart iTerm and the daemons; exclude /Volumes/Data/work (or the target/ dirs) from Spotlight.
- Consider a shared CARGO_TARGET_DIR or sccache instead of copying target/ per worktree.
- Old worktrees' target/ dirs (19) could be cleared for finished agents.
