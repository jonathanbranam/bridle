+++
id = "br-d99e"
title = "Stop hook: a finished worker must send its report and summary, not print them"
kind = "feature"
state = "planned"
created_at = "2026-09-29T11:30:25.975Z"
updated_at = "2026-09-29T11:30:30.860192Z"
+++

Goal: a worker cannot end its turn with a finished tree but no report. Overnight, Haiku workers often print their final bridle send done message instead of running it, then sit idle and the manager never learns they are done. Build on the existing Stop hook: crates/bridle/src/stop_check.rs, wiring in crates/bridle/src/commands.rs, docs/design/coordination.md (~line 107), docs/design/cli.md (stop-check, ~498), docs/spikes/05-stop-hook-findings.md. Today it only blocks on a claimed task with no thread entry since the claim. Change: also block when the claimed task has no task summary or no thread entry starting with done: AND the worktree is clean with commits ahead of the integration branch (it looks finished). The block reason is a direct instruction: run the summary command and the send command, do not print them. Keep the pure-function shape, unit-tested without a daemon; on any error allow the stop (existing rule); honour stop_hook_active as the existing code does. Add one sentence to workflow/base/skills/worker/SKILL.md handoff: run these commands, printing them does nothing. Acceptance: just check passes; unit tests for finished-no-report (block), finished-and-reported (allow), dirty tree (existing behaviour), error (allow). Model: Sonnet. Out of scope: daemon-side idle nudging; the check-on-stop task (touches the same files, run it after this one).
