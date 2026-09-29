+++
id = "br-14cd"
title = "Keep the warm build source fresh: build after each land, warm from there (b7cz step 2)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T21:19:30.262Z"
updated_at = "2026-09-29T21:19:33.960880Z"
size = "S"
+++

Ticket: docs/questions/open/build-cost-on-the-laptop-b7cz.md (read, incl. 'The warm cache goes stale' and the measurement section the previous task adds). Problem: workers warm their target/ from the main clone's target/ (crates/bridle-daemon/src/worktree.rs warm_target, on by default on macOS), which was last built 2026-09-28 and is never refreshed; and the integration worktree's target/ (<workspace>/integration, built at each land) will go stale too once qgma's land skips the check on a fast-forward (br-0e42). Build: 1) after every successful land (including a skipped-check fast-forward), in the daemon (crates/bridle-daemon/src/integrator.rs), run one incremental 'cargo build --workspace --all-targets' in the integration worktree in the background: off land's critical path (land returns first), niced, at most one at a time (a landing during a build queues one more, not many), failures logged and never failing land. Read the build command from config, not hardcoded for non-Rust projects: add [integration] warm_build (default unset = no background build; set to 'cargo build --workspace --all-targets' in this repo's .bridle/config.toml); keep it one small config key. 2) warm_target's source: prefer the integration worktree's target/ when it exists and its warm build is not running (or is newer than the clone's), else the clone's; never write to the source. Log which source and its age at spawn. 3) Docs: cli.md/agent-host docs for the config key and behaviour, b7cz ticket section, CHANGELOG. Acceptance: just check passes; tests: build is triggered after land and not blocking it, coalescing (two lands during one build make one more build), failure ignored, warm source selection prefers the fresh integration target. No live cargo in tests: inject the command (a fake script). Model: Sonnet. Out of scope: sccache, Linux (warm_target stays macOS-only), touching the human's clone.
