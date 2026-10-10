+++
id = "br-syqn"
title = "Ticket fields get standard names (blocked_by, related, parent, created, created_by, resolved) and a theme field, read in both old and new forms (syqn part 1)"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T19:47:21.170Z"
updated_at = "2026-10-10T22:01:03.513143Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/wsyqn"
commit = "1a64bb8e83c8b9270b92085cea3db184f50e79e5"
summary = "Ticket frontmatter fields get standard names (crates/bridle/src/ticket.rs, cli.rs): needs->blocked_by, see->related, opened->created, filed_by->created_by, closed->resolved, plus optional parent (bare or <prefix>-<id>, checked well-formed only) and theme (slug; ticket new --theme, ticket set theme). Readers (ticket check) accept both forms, new wins, both present warns; writers (new/set/resolve) write only new names; ticket set maps old field names to new and drops the old-named line. ticket new flags --blocked-by/--related keep --needs/--see as aliases. Docs updated (README, cli.md, tickets rule, CHANGELOG). Migration: none in this task; existing tickets keep working and bridle ticket check stays clean. Part 2/3 not done. just check green (1468 tests). Note: an earlier full run had one load-sensitive failure (lifecycle_test spawn_child_orphan_is_swept_on_stop) that passes alone and on rerun."
ticket = "syqn"
+++

Ticket: docs/tickets/open/ticket-fields-get-standard-names-blocked-by-related-parent-c-syqn.md (read it fully, and br-22ab's ticket: sections 7 (links) and 11 (migration, step 1) and the Plan's note on themes). Model: Sonnet. This is part 1 of 3: field names and theme. Part 2 (project-qualified IDs) and part 3 (the migration that renames existing tickets) are separate tasks that follow; do not do them.

Build:
1. Renames, decided by the human: `needs` -> `blocked_by`, `see` -> `related`, `opened` -> `created`, `filed_by` -> `created_by`, `closed` -> `resolved`; new `parent` (one ID); `kind`, `repos`, `tasks`, `changes`, `specs` unchanged. Find the one place that parses and writes ticket frontmatter (grep for `filed_by` and `needs` in crates/; bridle ticket new/set/resolve/check, and any reader: the docs crate, gateway, bridle-spec if it reads tickets) and make every reader accept BOTH the old and the new names (the new name wins if both are present; `ticket check` warns on a ticket holding both). Writers (`bridle ticket new/set/resolve`, and the code stamping `closed:`/`resolved:`) write the NEW names only. `bridle ticket set` accepts both old and new field names as the field argument, mapping old to new.
2. New field `theme: <slug>` (lowercase letters, digits, hyphens; no registry; `ticket check` rejects a malformed slug and accepts any well-formed one; the enforced list comes later with products, epic g5dm). `bridle ticket new --theme <slug>` and `ticket set <id> theme <slug>`.
3. `parent: <ID>` accepts a bare ID or, once part 2 lands, a project-qualified one: for now store and check what is given as a well-formed ID (letters/digits with optional `<prefix>-`), do not resolve across projects.
4. Docs in step: docs/README.md (ticket conventions), docs/design/cli.md, the `tickets` rule (workflow/base/rules/tickets.md), CHANGELOG.md. Do NOT rewrite existing tickets in this repo; part 3's migration does that, so keep `bridle ticket check` clean on today's old-format tickets.
Tests: old-format ticket reads; new-format reads; both present warns and new wins; writers emit new names; set maps old to new; theme validation; resolve stamps `resolved:`.
Acceptance: just check passes; `bridle ticket check` still clean on this repo.
Migration: none in this task (reads both forms, so nothing breaks before part 3 runs everywhere); say so in the task comment.
Out of scope: qualified IDs (part 2), the rename migration (part 3), ticket types and readiness (step 2), indexing links (step 3), dropping `changes`/`specs`.

## Thread

### note · external:advisor/product-manager · 2026-10-09T19:48:05.158Z
advisor (product-manager): readied with the human's approval of the 22ab plan (2026-10-09 ~4:40 PM ET). Normal priority: after machine setup. Steps 2-4 (br-bpku, br-3v75, br-72t9) follow in order.

### question · external:advisor/product-manager · 2026-10-09T19:56:32.162Z
Do themes and epics cross projects? See the ticket's 'Open question' section (the PdM's recommendation: themes are the human's, same slug in every project, no registry; an epic lives in one project with children anywhere). Waiting on the human before planning.

### answer · external:advisor/product-manager · 2026-10-09T20:07:12.112Z
From the human, via advisor (product-manager), 2026-10-09 ~5:20 PM ET: some themes and epics cross projects, some don't; keep the design flexible. New concept: a product (a set of projects with one roadmap and one PdM), epic br-g5dm, designed later. For this step: theme: accepts any well-formed slug, no registry or product scoping; parent takes project-qualified IDs. See the ticket's 'Open question' section.

### note · system · 2026-10-10T00:19:11.133Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · external:advisor/product-manager · 2026-10-10T00:19:24.598Z
advisor (product-manager): readied again after the 4 h stale return. pm-1: please plan it now (the question is answered above); planning doesn't jump the build queue. It's step 1 of epic everything-is-a-ticket (epic 2), after machine setup.

### note · agent:pm-1 · 2026-10-10T00:19:46.891Z
pm-1: split in three so each merges alone. This task is part 1 (names, theme, read both forms). Part 2 (project-qualified IDs, section 8 of 22ab) and part 3 (the xebc migration across projects, trial branches only per the existing-projects rule) I will file once part 1 lands, so their briefs can use its parser; not filed now (no need yet).

### note · agent:wsyqn · 2026-10-10T21:57:46.530Z
No migration in this task: readers accept both forms, so nothing breaks before part 3 runs. ticket check on this repo reports 34 task-link problems (tasks names ... not a task made from a ticket); none concern field names, and none are from this change.

### note · agent:wsyqn · 2026-10-10T21:57:49.394Z
done: ticket fields renamed (blocked_by/related/created/created_by/resolved) + parent + theme, both forms read, new written; just check exit 0, 1468 tests; df0df4a1 + merge b55f5e31 (check ran on b55f5e31). Note: ticket check shows 34 pre-existing task-link problems, none about field names.

### note · agent:manager-2 · 2026-10-10T22:01:03.513Z
integrated: 1a64bb8e83c8b9270b92085cea3db184f50e79e5 (branch bridle/wsyqn)
