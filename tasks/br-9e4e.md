+++
id = "br-9e4e"
title = "Clean bridle's own tickets so 'bridle ticket check' passes (7gk7 follow-up)"
kind = "chore"
state = "integrated"
created_at = "2026-09-30T20:12:31.211Z"
updated_at = "2026-09-30T20:21:19.450344Z"
branch = "bridle/ticket-clean"
commit = "cc893d0188c7099705d010e427d0b7e8b87aab68"
summary = "Bridle's tickets now pass `bridle ticket check` (exit 0). Checker fix: `[[...]]` inside inline code is ignored (crates/bridle/src/ticket.rs, test extended). Backfilled `closed` on 78 resolved tickets from the last git commit touching each file (UTC; one-off script, not committed). hb0q, rl2v and f1ky weren't a format problem in the name: their ids contain characters outside the id alphabet (0, l, 1), so they were reminted: hb0q->dv9u, rl2v->4f34, f1ky->5heh; files renamed, frontmatter id and wiki links/see entries updated (usage-and-budget.md, mt7r, g3ck, agent-host brief). Prose mentions of the old ids (orchestrator-history, a test comment, a ticket body) were left as history. Dangling needs/see: repointed kc4v -> context-tokens-overcounts-multi-call-turns-kc4v and mrhe -> bridle-without-a-clone-of-its-repo-mrhe. Removed: `existing-projects` (a rule) from push-the-state-branch-we2r; `d99e` from routine-notes-reach-the-humans-inbox-hx7t; `u7pw` (a spike) from allowed-warning-overrides-the-override-kv7d; `incidents` from budget-resume-skips-the-manager-k7nr."
+++

Workers are now told to run 'bridle ticket check' when a ticket changes, and it fails on bridle's own docs/tickets with 95 problems (full list in the br-ae32 task summary, or run it). Make it exit 0. (1) Checker bug first: [[focus]], [[budget.schedule]] and similar TOML table names in fenced code blocks or inline code are not links; make the link check skip fenced blocks and inline code (crates/bridle ticket module), with a test. (2) Backfill closed: on the ~78 resolved tickets that lack it: use the date-time of the git commit that moved/last touched the file (git log -1 --format=%cI -- <path>), falling back to opened; ISO UTC as 'bridle ticket resolve' writes it; do this with a small one-off script or command run once, not a new feature; don't commit the script. (3) The 3 files with no valid id tail (hb0q, rl2v, f1ky): rename to <descriptive-tail>-<id>.md using the id in their frontmatter (mint one with bridle ticket new's logic only if the frontmatter has none) and update references to the old stems across docs/ (grep the old stem). (4) Dangling needs/see (kc4v, mrhe, f1ky and others): repoint to the right ticket when obvious (renamed or resolved tickets), else remove the entry; list what you removed in your summary. Change no ticket body text beyond links and frontmatter. Docs only plus the one checker fix; CHANGELOG line. Acceptance: bridle ticket check exits 0 on docs/tickets; just check passes. Model: Sonnet. Out of scope: spikes folder, other projects' tickets, new checker rules.

## Thread

### note · agent:ticket-clean · 2026-09-30T20:21:14.583Z
done: bridle ticket check exits 0 on docs/tickets; checker skips inline code; closed backfilled on 78; hb0q/rl2v/f1ky had invalid-alphabet ids so reminted (dv9u/4f34/5heh); dangling needs/see fixed (removals in summary); main merged, just check green (923 passed); 6bb305d

### note · agent:manager-2 · 2026-09-30T20:21:19.450Z
integrated: cc893d0188c7099705d010e427d0b7e8b87aab68 (branch bridle/ticket-clean)
