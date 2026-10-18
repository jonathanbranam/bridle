+++
id = "br-ubjd"
title = "A CHANGELOG line for every landed task, written on the branch, and one section per kind under Unreleased"
kind = "chore"
state = "integrated"
created_at = "2026-10-09T00:54:15.168Z"
updated_at = "2026-10-18T12:09:28.021783Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
    "agent:wubjd",
    "agent:wubjd2",
]
branch = "bridle/wubjd2"
commit = "b2375c7cd7f67f80a07d2c2d18d7351363223fb8"
summary = "Remainder: CHANGELOG.md Unreleased is now ### Added / Changed / Fixed / Internal (existing lines moved by prefix); worker.md, manager.md and docs-current say which heading and to insert at the top of the section; cli.md updated. integrator.rs added_under_unreleased now requires the added line to sit under one of the four headings (a bare heading or a line under no/unknown heading does not count); tests per heading and for none; land fixtures updated. Caveat: when main adds flat lines, a merge may drop them into the wrong section; I re-filed them by prefix. No merge driver."
ticket = "ubjd"
+++

docs/tickets/open/a-changelog-line-for-every-landed-task-written-on-the-branch-ubjd.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.316Z
watching the task

### note · external:advisor/product-manager · 2026-10-17T15:26:08.058Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wubjd · 2026-10-18T09:17:03.636Z
built: d71643e77ec3023d09e2d2849f6d451042264432 (branch bridle/wubjd)

### note · agent:wubjd · 2026-10-18T09:17:03.875Z
done: land refuses a branch with no CHANGELOG line under Unreleased (question/research/explore/incident and docs/tickets-only exempt); roles/rule/cli.md updated; just check exit 0, 1866 tests passed; d71643e77ec3023d09e2d2849f6d451042264432

### note · agent:manager-2 · 2026-10-18T10:02:27.962Z
integrated: 6c0a50e69989813418c312b85e947af571dd7cda (branch bridle/wubjd)

### note · agent:manager-2 · 2026-10-18T10:02:50.879Z
cleanup: removed agent wubjd, branch bridle/wubjd

### note · external:advisor/product-manager · 2026-10-18T10:03:17.240Z
advisor product-manager: verified on main (6c0a50e6 = worker's d71643e7): items 1 (a line on the branch for every task) and 2 (land refuses without one, exemptions) are done. NOT done: item 3 and the Internal heading. CHANGELOG.md's Unreleased is still a flat list ('- Fix (...)', '- Change (...)'), with no '### Added / ### Changed / ### Fixed / ### Internal' sections. Ticket stays open. pm-1: plan a follow-up task here. Approach: restructure Unreleased into one section per kind (Added, Changed, Fixed, Internal; tests/docs/research under Internal); the worker and manager role text says which heading a line goes under; land check accepts a line under any of them. Propose a fixed insertion point (top of each section) so conflicts stay rare. Chore, normal priority.

### note · agent:pm-1 · 2026-10-18T10:03:25.938Z
Remainder brief (pm-1), item 3 and the Internal heading. Model: Sonnet (small). Items 1-2 are landed (6c0a50e6); do not redo.
(1) Restructure CHANGELOG.md "Unreleased" from the flat list into one section per kind: ### Added, ### Changed, ### Fixed, ### Internal (tests, docs, research, role text go under Internal). Move the existing lines under the right heading by their prefix (Fix -> Fixed, Change -> Changed, etc.); keep each line's text.
(2) Role text (workflow/base/roles/worker.md and manager.md, and the docs-current rule if it names the format) says which heading a line goes under and to insert at the TOP of its section, so concurrent branches conflict rarely.
(3) The land check accepts a line under any of the four headings (extend the check from d71643e7); test for a line under each heading and for none.
CHANGELOG line, cli.md only if wording changes. Acceptance: just check. Out of scope: a git merge driver (not yet; fixed insertion point first).

### note · agent:wubjd2 · 2026-10-18T12:09:12.970Z
built: 5b6ef4183c90ee43a300f221b60cf9595140876e (branch bridle/wubjd2)

### note · agent:wubjd2 · 2026-10-18T12:09:15.987Z
done: CHANGELOG Unreleased in 4 sections, roles/rule/cli.md say heading + top insertion, land accepts a line only under one of them (tests per heading and none); just check exit 0, 1870 tests passed; main merged in; 5b6ef4183c90ee43a300f221b60cf9595140876e

### note · agent:manager-2 · 2026-10-18T12:09:28.021Z
integrated: b2375c7cd7f67f80a07d2c2d18d7351363223fb8 (branch bridle/wubjd2)
