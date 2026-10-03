+++
id = "br-5c6c"
title = "Explain the gap between 19K measured and 48-60K live starting context (ct8m step 3)"
kind = "research"
state = "integrated"
created_at = "2026-09-29T17:59:36.068Z"
updated_at = "2026-09-29T18:19:52.823477Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
branch = "bridle/ctx-gap"
commit = "344eb74"
summary = "Explained the gap in docs/spikes/08-lean-context-findings.md (new section, no prompt edits, no live run: transcripts already hold the data). A real Sonnet worker's first call is 17.5K (77 workers), matching the spike; it grows ~1.4K/turn to a median 53K after ~27 turns, which is the 48-60K: ~19K reads (source/docs via cat/sed dominate), ~15K the agent's own tool calls and thinking, 17.5K fixed. No hidden fixed cost. Real fixed-cost finding: manager-2 and pm-1 started at 32K because they predate c8f5baa (no --setting-sources project), so they loaded the human's 4 plugins and 31 skills; a restart under current command.rs fixes ~15K each. Ranked recommendations: restart long-lived roles, --tools whitelist, tell workers to read by range (CHANGELOG, cli.rs, docs/design/* shotgun cats), cap check/git output, compact long-lived roles; CLAUDE.md/preamble duplication is only ~250 tokens. Caveat: byte-to-token split is an estimate (2.7 B/token); thinking is over-counted (signature)."
+++

Ticket: docs/questions/open/*ct8m.md, plan step 3; docs/spikes/08-*.md (landed dfd3fa8; its section on the gap is your starting point). Goal: explain, from real data, where the 48-60K in bridle agents comes from when a scratch spawn's first turn is ~19K. Method: take a few recent real spawns (worker, manager, product-manager) from the daemon's events and transcripts under ~/.claude/projects for this repo, and split each first turn into: system prompt and tools (fixed), appended preamble and role file, CLAUDE.md, files the agent read at once (bridle prime output, role docs, task brief), tool results. A cheap Haiku live run in a scratch dir (under 1 USD) may reproduce a spawn's first turn with the real appended prompt if it helps. Then propose, ranked by tokens saved, what to trim (repeated text between preamble, role file, CLAUDE.md and prime output; what can become a pointer). Deliverable: append a section to docs/spikes/08 findings or a new docs/spikes/NN file, ending with a short ranked recommendation list. Docs only; do not edit prompts. Acceptance: just check passes. Model: Sonnet. Out of scope: implementing any trim (a follow-up build task), the orchestrator and advisor scripts.

## Thread

### note · agent:ctx-gap · 2026-09-29T18:19:36.551Z
done: gap explained, no hidden fixed cost (workers 17.5K first call, grow ~1.4K/turn to ~53K from reads and own output); manager-2/pm-1 started at 32K only because they predate c8f5baa, restart fixes ~15K; ranked list in spike; docs: docs/spikes/08-lean-context-findings.md; a40c06d

### note · agent:manager-2 · 2026-09-29T18:19:42.443Z
integrated: 344eb74 (branch bridle/ctx-gap)

### note · agent:manager-2 · 2026-09-29T18:19:52.823Z
cleanup: removed agent ctx-gap, branch bridle/ctx-gap
