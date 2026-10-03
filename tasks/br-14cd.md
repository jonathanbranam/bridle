+++
id = "br-14cd"
title = "Keep the warm build source fresh: build after each land, warm from there (b7cz step 2)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T21:19:30.262Z"
updated_at = "2026-09-30T11:04:02.924278Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
+++

Ticket: docs/questions/open/build-cost-on-the-laptop-b7cz.md (read, incl. 'The warm cache goes stale' and the measurement section the previous task adds). Problem: workers warm their target/ from the main clone's target/ (crates/bridle-daemon/src/worktree.rs warm_target, on by default on macOS), which was last built 2026-09-28 and is never refreshed; and the integration worktree's target/ (<workspace>/integration, built at each land) will go stale too once qgma's land skips the check on a fast-forward (br-0e42). Build: 1) after every successful land (including a skipped-check fast-forward), in the daemon (crates/bridle-daemon/src/integrator.rs), run one incremental 'cargo build --workspace --all-targets' in the integration worktree in the background: off land's critical path (land returns first), niced, at most one at a time (a landing during a build queues one more, not many), failures logged and never failing land. Read the build command from config, not hardcoded for non-Rust projects: add [integration] warm_build (default unset = no background build; set to 'cargo build --workspace --all-targets' in this repo's .bridle/config.toml); keep it one small config key. 2) warm_target's source: prefer the integration worktree's target/ when it exists and its warm build is not running (or is newer than the clone's), else the clone's; never write to the source. Log which source and its age at spawn. 3) Docs: cli.md/agent-host docs for the config key and behaviour, b7cz ticket section, CHANGELOG. Acceptance: just check passes; tests: build is triggered after land and not blocking it, coalescing (two lands during one build make one more build), failure ignored, warm source selection prefers the fresh integration target. No live cargo in tests: inject the command (a fake script). Model: Sonnet. Out of scope: sccache, Linux (warm_target stays macOS-only), touching the human's clone.

## Thread

### note · agent:pm-1 · 2026-09-29T21:27:04.323Z
HELD (m-2055): no cache work until the measurement (br-656b) says a fresh warm cache saves enough. Not queued.

### note · agent:pm-1 · 2026-09-30T05:49:36.671Z
PM 2026-09-30: UNHELD. The measurement (br-656b) says warm from a fresh integration target takes a build from 3m40s to 1m39s, and a stale clone target gains nothing, so keeping the warm source fresh pays. Ticket paths in the brief: docs/questions is now docs/tickets.

### note · agent:pm-1 · 2026-09-30T05:50:11.582Z
PM 2026-09-30: HELD again (orchestrator): the human said measure, then decide before any cache work; the recommendation is theirs to accept. Not queued.

### note · agent:pm-1 · 2026-09-30T05:50:20.118Z
HELD again per the human/orchestrator: do NOT start br-14cd (the queue can't be emptied, so it still shows). Wait for the human's decision on cache work.

### note · agent:pm-1 · 2026-09-30T11:04:02.924Z
PM HANDOFF 2026-09-30 (pm-1 renewing): state. Queue = only br-14cd in tier 1, HELD for the human's decision on cache work (orchestrator will ask them; the queue can't be emptied, manager-2 told not to start it). Landed/claimed since: NUC A/B/C, hw6c 1-2, mail 1-3, k6b3, mrhe 1-2, ervd 2, fr6q (br-d87c), 24mj part 1 (br-33b3) (last two: no longer in the queue, not verified landed). br-9307 dropped-as-completed (meta-notes main 5b65a89 v2.0.0). HELD on the human: a67t/6rh7 (br-163f, br-146a were NOT built; only tickets), meta-notes executable scenarios (low priority), track-web real trial, life assistant (br-ca8a), email go-live (br-fc05 theirs), refinement design (hvxk/k7tm), xe5c planning, otters/file-db surveys, cross-project budget/provenance/remote daemons design. Lessons: match a task to the commit that IMPLEMENTS it, not its ticket commit; 'task done' refuses commits from other repos (drop with a reason instead); command substitution and heredocs in Bash get denied, use plain commands; send to the orchestrator as external:orchestrator. Next: wait for orchestrator messages; refill the queue only with decision-free startable work.
