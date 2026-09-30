+++
id = "br-9e4e"
title = "Clean bridle's own tickets so 'bridle ticket check' passes (7gk7 follow-up)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T20:12:31.211Z"
updated_at = "2026-09-30T20:12:32.754798Z"
+++

Workers are now told to run 'bridle ticket check' when a ticket changes, and it fails on bridle's own docs/tickets with 95 problems (full list in the br-ae32 task summary, or run it). Make it exit 0. (1) Checker bug first: [[focus]], [[budget.schedule]] and similar TOML table names in fenced code blocks or inline code are not links; make the link check skip fenced blocks and inline code (crates/bridle ticket module), with a test. (2) Backfill closed: on the ~78 resolved tickets that lack it: use the date-time of the git commit that moved/last touched the file (git log -1 --format=%cI -- <path>), falling back to opened; ISO UTC as 'bridle ticket resolve' writes it; do this with a small one-off script or command run once, not a new feature; don't commit the script. (3) The 3 files with no valid id tail (hb0q, rl2v, f1ky): rename to <descriptive-tail>-<id>.md using the id in their frontmatter (mint one with bridle ticket new's logic only if the frontmatter has none) and update references to the old stems across docs/ (grep the old stem). (4) Dangling needs/see (kc4v, mrhe, f1ky and others): repoint to the right ticket when obvious (renamed or resolved tickets), else remove the entry; list what you removed in your summary. Change no ticket body text beyond links and frontmatter. Docs only plus the one checker fix; CHANGELOG line. Acceptance: bridle ticket check exits 0 on docs/tickets; just check passes. Model: Sonnet. Out of scope: spikes folder, other projects' tickets, new checker rules.
