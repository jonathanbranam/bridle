+++
id = "br-e5f1"
title = "Self-upgrade: wake only on what needs attention, record every step as an event"
kind = "feature"
state = "planned"
created_at = "2026-10-03T12:25:23.496Z"
updated_at = "2026-10-03T13:08:11.627788Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

original id: p372
Goal: self-upgrade wakes only on what needs attention, and every step is an event. Full spec: docs/tickets/open/self-upgrade-wake-only-on-what-needs-attention-record-every-p372.md (Proposal items 1-4; human agreed 2026-10-03).
Files: crates/bridle-daemon/src/server.rs (upgrade_in_background), wake.rs, crates/bridle-api/src/types.rs (event kinds upgrade.*), docs/design/agent-host/daemon.md, api.md, storage.md if events are listed, CHANGELOG.
Acceptance: just check passes; tests: skipped commit => event and no wake; no-quiet-point give-up => no human note and a later retry; real failures still wake upgrade_failed and tell the human.
Migration: none (no project files change; new event kinds only).
Model: Sonnet. Out of scope: self_upgrade=release (br-88d4), workflow checkout (br-751e).
