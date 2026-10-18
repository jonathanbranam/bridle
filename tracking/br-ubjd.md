+++
id = "br-ubjd"
title = "A CHANGELOG line for every landed task, written on the branch, and one section per kind under Unreleased"
kind = "chore"
state = "built"
created_at = "2026-10-09T00:54:15.168Z"
updated_at = "2026-10-18T09:17:03.875329Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
    "agent:wubjd",
]
branch = "bridle/wubjd"
commit = "d71643e77ec3023d09e2d2849f6d451042264432"
summary = "bridle ticket land now refuses a squash that adds no line under '## Unreleased' in CHANGELOG.md (integrator.rs changelog_missing; exempt: question/research/explore/incident kinds, taken from the row via LandInput.changelog_exempt, and changes confined to docs/tickets/). Role texts (manager, worker) and rule docs-current say every landed task gets a line, on the branch; cli.md notes the check and defers a merge driver and per-kind sections (entries are flat bullets; YAGNI). Existing land tests now give their branches a CHANGELOG line; new test covers refuse and exempt."
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
