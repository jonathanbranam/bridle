# Orchestrator state

Current state only, for the next orchestrator session. The role is in
`workflow/base/roles/orchestrator.md`; past sessions' notes and the full decisions log are in
`docs/context/orchestrator-history.md` (read on demand, not at startup). Keep this file short
(ticket ct8m): replace, don't append. Last updated 2026-09-29 17:50 UTC (1:50 PM), at the
handover from the tenth session.

## Running

- **bridle:** `pm-1` (product manager, ~89K), `manager-2` (development manager, ~82K; renew
  either above ~140K). Worker `lean-spike` on br-60ec (ct8m step 1, the starting-context spike;
  cheap Haiku live runs authorised, a few dollars). br-9fca (ct8m step 2, minimal toolsets)
  waits on it.
- **meta-notes, track-web:** managers idle, waiting on the human's tasks. Check with
  `bridle agents --project <p>` on heartbeats; the watcher covers bridle only.
- `max_workers` is 2 (human-only to change).

## Identity and tokens

`scripts/claude-orchestrator` sets `BRIDLE_AS=orchestrator`; tokens for all three projects are
in `~/.bridle/credentials.toml` (t6kq, done 2026-09-29). The old `~/.bridle-*.token` files are
gone. The watcher needs no token env: `scripts/orchestrator-watch.sh <seq>`.

## In flight and next

- **ct8m (lean context), the human's priority.** Steps: spike (running) → background agents'
  toolsets (br-9fca) → trim the appended prompt → the orchestrator and advisor launch scripts
  (keep Remote Control; ask the human whether to keep AskUserQuestion) → track growth.
  Measured this session: the orchestrator starts at ~50K (~40K Claude Code's prompt and tools,
  ~9K the prime) and grew ~1.75K/min while busy (134K at 48 min).
- **fx7x (the daemon supervises the orchestrator).** Design landed (277e495,
  `docs/design/agent-host/orchestrator-supervision.md`, spike 07). Next: pm-1 queues the build
  slices. The human's settled answers: bridle types into the pane only to relaunch when no
  `claude` runs; wakes stay in-session via one bridle wait command; forced restart = hand over,
  else stop at the deadline and relaunch; crash-loop backoff in scope; the pane is found by its
  tmux tag `@bridle=orchestrator` (pane `%39`, `pi:5.4`, tagged); thresholds 150K note / 210K
  plan a handover / 255K hand over now.
- **Held with the human:** one design for incidents (nc7r) and human to-dos (ex9q): "things
  asked of someone that may resolve without them". The incidents doc landed (c086457); its
  build split and br-c83e (to-dos; WIP 648321e on its branch, worker removed) wait for the
  human. My recommendation: one "open request" record with an owner, a way to resolve it and an
  optional check that closes it automatically.
- **For the human to read:** the briefs `docs/briefs/specs.md`, `tasks.md` and the agent host
  brief (8awb parts 1-3); P6 (specs migrations) waits on their approval. Also the launchd plan's
  upgrade options (79aac96).
- **Design discussions, not builds:** hvxk (refining a task with the human), k7tm (tickets vs
  tasks), 9mxw (per project/machine/account).

## This session (tenth, 16:48-17:50 UTC)

- Restarted after the previous session died at 12:36 UTC with no warning (fx7x). The human's
  SSH key and Tailscale were down; fixed. I may now `git push origin main`
  (`.claude/settings.local.json`).
- Landed and CI-green: incidents design, specs/tasks/agent-host briefs, docs-current rule
  (3ndf), fx7x design. br-b966 (questions reach their addressee) and the last four commits were
  still in CI at handover: check `gh run list --branch main` first.
- The watcher ignores my own open questions (969a361) and uses credentials.toml (9b87047).

## For the next maintenance window (budget hold)

- Rebuild (`cargo install --path crates/bridle`) and ask the human for one restart: fx7x 1a
  (9368d8d) changed `scripts/claude-orchestrator` to a SessionStart hook that calls
  `bridle orchestrator note-session`, which the installed binary lacks. Until the rebuild, a
  relaunched orchestrator has no context wake (the hook fails; the session still starts).
  `[orchestrator] enabled` stays off until the human opts in. The same rebuild brings ct8m
  step 2's lean toolsets (f6a0b75); after it, check a fresh worker's first-turn size and that
  research workers don't miss WebFetch/WebSearch (per-task tools: br-abc3).

## Watch

- Managers sometimes don't push after a merge; check `git status -sb` after main moves.
- Haiku workers sometimes print their report instead of sending it.
- The watcher wakes on my own commits to main; that's noise, restart it.
