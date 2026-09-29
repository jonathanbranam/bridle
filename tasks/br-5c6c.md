+++
id = "br-5c6c"
title = "Explain the gap between 19K measured and 48-60K live starting context (ct8m step 3)"
kind = "research"
state = "planned"
created_at = "2026-09-29T17:59:36.068Z"
updated_at = "2026-09-29T17:59:38.912631Z"
size = "S"
+++

Ticket: docs/questions/open/*ct8m.md, plan step 3; docs/spikes/08-*.md (landed dfd3fa8; its section on the gap is your starting point). Goal: explain, from real data, where the 48-60K in bridle agents comes from when a scratch spawn's first turn is ~19K. Method: take a few recent real spawns (worker, manager, product-manager) from the daemon's events and transcripts under ~/.claude/projects for this repo, and split each first turn into: system prompt and tools (fixed), appended preamble and role file, CLAUDE.md, files the agent read at once (bridle prime output, role docs, task brief), tool results. A cheap Haiku live run in a scratch dir (under 1 USD) may reproduce a spawn's first turn with the real appended prompt if it helps. Then propose, ranked by tokens saved, what to trim (repeated text between preamble, role file, CLAUDE.md and prime output; what can become a pointer). Deliverable: append a section to docs/spikes/08 findings or a new docs/spikes/NN file, ending with a short ranked recommendation list. Docs only; do not edit prompts. Acceptance: just check passes. Model: Sonnet. Out of scope: implementing any trim (a follow-up build task), the orchestrator and advisor scripts.
