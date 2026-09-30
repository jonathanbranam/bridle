+++
id = "br-ae32"
title = "bridle ticket set / check, and the ticket rule for every project (7gk7 B)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T19:49:01.127Z"
updated_at = "2026-09-30T19:52:37.372726Z"
+++

Slice B of 7gk7 (br-01ff). Needs slice A merged (reuse its ticket module for frontmatter and lookup). Read docs/tickets/open/tickets-through-the-bridle-binary-7gk7.md. Goal: (1) 'bridle ticket set <id> <field> <value>' edits frontmatter (title, repos, changes, specs, needs, see; list fields accept comma separated values; refuses id/opened/closed and unknown fields). (2) 'bridle ticket check [--quiet]': every ticket in docs/tickets/{open,resolved} has the required frontmatter (id, title, opened, repos, changes, specs, needs, see; closed only under resolved/ and required there), id matches the file name tail and is unique, needs/see name existing tickets (by id or full stem), [[stem|text]] links in bodies point at existing files; nonzero exit and a readable list of problems. Must pass on bridle's own docs/tickets as they are today or report only real problems (fix none of them; list in your summary). (3) A rule file workflow/base/rules/tickets.md (shipped to every bridle project by bridle sync; look at how other base rules are written, keep it short): tickets live in docs/tickets/open|resolved, are created/edited/resolved only via bridle ticket, write the body freely; and a line in the worker/manager role text to run 'bridle ticket check' when a ticket changes. Tests: each check failure, set happy path and refusals. Docs: cli.md, docs/README.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: templates, spikes folder, migrating other projects' existing tickets (each project's own adoption, needs the human), p2ys/k7tm/xe5c.

## Thread

### note · agent:pm-1 · 2026-09-30T19:52:37.372Z
PM: keep file names <descriptive-tail>-<id>.md (human, for now). Your new rule workflow/base/rules/tickets.md should point at the existing workflow/base/rules/ticket-references.md (name tickets to the human by file-name stem) rather than repeat it.
