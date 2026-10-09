+++
id = "br-pakx"
title = "Specs: a way to run executable specs in CI without a bridle checkout"
kind = "feature"
state = "pending"
created_at = "2026-10-05T02:51:25.551Z"
updated_at = "2026-10-09T11:04:38.316004Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
ticket = "pakx"
+++

docs/tickets/open/specs-a-way-to-run-executable-specs-in-ci-without-a-bridle-c-pakx.md

submitted by external:orchestrator@nuc

vitest-bridle (and the pytest plugin) get scenarios from `bridle spec export --format json` at collection time, so they need a bridle binary. A project's GitHub CI doesn't have one, so the spec tests skip there and CI never runs the specs. Workers run them before merge, but a regression can still reach main uncaught. Options: a released bridle binary plus a setup-bridle GitHub Action; or a committed, checked export (`bridle spec export` output kept in the repo, with `bridle spec check` failing when it's stale) so CI needs only vitest.

From the meta-notes-ui project (orchestrator, 2026-10-05), adopting bridle specs with vendored vitest-bridle (mu-943b, mu-hzf9). Local log: meta-notes-ui docs/tickets/open (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.609Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:51:56.253Z
Accepted. Ticket pakx minted (docs/tickets/open/specs-a-way-to-run-executable-specs-in-ci-without-a-bridle-c-pakx.md); br-m4cx merged into it. It recommends a committed, checked export plus a format-version guard in the test runners, deferring a released binary and npm publishing. Waiting on the human's okay before this is readied and planned.

### note · external:advisor/product-manager · 2026-10-09T11:04:38.316Z
watching the task
