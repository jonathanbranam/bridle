+++
id = "br-ae32"
title = "bridle ticket set / check, and the ticket rule for every project (7gk7 B)"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T19:49:01.127Z"
updated_at = "2026-09-30T20:12:17.719593Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/ticket-b"
commit = "2fbb2cd7f09f30bafa92035e5b2fbd51d6d3736e"
summary = "Slice B of 7gk7. `bridle ticket set <id> <field> <value>` (title or list fields, comma-separated; refuses id/opened/closed and unknown fields; works on open and resolved; closed stays last) and `bridle ticket check [--quiet]` (required frontmatter, closed only/required under resolved/, id = file-name tail and unique, needs/see name an existing id or stem, [[links]] outside code fences resolve by stem under docs/ or as a path from repo root/docs/) in crates/bridle/src/ticket.rs; problems to stderr, exit 1. New rule workflow/base/rules/tickets.md (points at ticket-references), a line in worker and manager roles, cli.md, docs/README.md, CHANGELOG. Tests cover each check failure, set happy path and refusals. Decision: links to non-file targets count as problems. `bridle ticket check` on bridle's own tickets reports 95 real problems, none fixed: 78 resolved tickets lack `closed:`; 3 file names lack a valid id tail (hb0q-ticket how-does-bridle-budget-hold..., rl2v sync-does-not-render-role-prompts, f1ky two-flaky-test-failures, whose id has '1' outside the alphabet); dangling needs/see (kc4v accurate-context, mrhe bridle-without-a-local-clone, existing-projects, d99e, u7pw, incidents, f1ky); dangling [[focus]] / [[budget.schedule]] links (TOML names, not files) in cvaq, 6t29, n9qh."
+++

Slice B of 7gk7 (br-01ff). Needs slice A merged (reuse its ticket module for frontmatter and lookup). Read docs/tickets/open/tickets-through-the-bridle-binary-7gk7.md. Goal: (1) 'bridle ticket set <id> <field> <value>' edits frontmatter (title, repos, changes, specs, needs, see; list fields accept comma separated values; refuses id/opened/closed and unknown fields). (2) 'bridle ticket check [--quiet]': every ticket in docs/tickets/{open,resolved} has the required frontmatter (id, title, opened, repos, changes, specs, needs, see; closed only under resolved/ and required there), id matches the file name tail and is unique, needs/see name existing tickets (by id or full stem), [[stem|text]] links in bodies point at existing files; nonzero exit and a readable list of problems. Must pass on bridle's own docs/tickets as they are today or report only real problems (fix none of them; list in your summary). (3) A rule file workflow/base/rules/tickets.md (shipped to every bridle project by bridle sync; look at how other base rules are written, keep it short): tickets live in docs/tickets/open|resolved, are created/edited/resolved only via bridle ticket, write the body freely; and a line in the worker/manager role text to run 'bridle ticket check' when a ticket changes. Tests: each check failure, set happy path and refusals. Docs: cli.md, docs/README.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: templates, spikes folder, migrating other projects' existing tickets (each project's own adoption, needs the human), p2ys/k7tm/xe5c.

## Thread

### note · agent:pm-1 · 2026-09-30T19:52:37.372Z
PM: keep file names <descriptive-tail>-<id>.md (human, for now). Your new rule workflow/base/rules/tickets.md should point at the existing workflow/base/rules/ticket-references.md (name tickets to the human by file-name stem) rather than repeat it.

### note · agent:ticket-b · 2026-09-30T20:12:02.816Z
done: bridle ticket set/check + tickets rule; check green (923 tests); own tickets show 95 real problems, listed in summary; 70a0493

### note · agent:manager-2 · 2026-09-30T20:12:07.578Z
integrated: 2fbb2cd7f09f30bafa92035e5b2fbd51d6d3736e (branch bridle/ticket-b)

### note · agent:manager-2 · 2026-09-30T20:12:17.719Z
cleanup: removed agent ticket-b, branch bridle/ticket-b
