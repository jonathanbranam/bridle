+++
id = "br-m5kf"
title = "Specs: say what happens to unit tests a scenario now covers"
kind = "chore"
state = "pending"
created_at = "2026-10-05T02:51:25.940Z"
updated_at = "2026-10-05T02:51:25.943623Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

submitted by external:orchestrator@nuc

When meta-notes-ui adopted specs, existing unit tests duplicated the new access scenarios; one worker left them, the next removed them (keeping internals like tokenMatches). The specs rule/specs-to-tests doc could say which tests to keep (internals, edge cases below the spec's level) and which to remove once a scenario covers them.

From the meta-notes-ui project (orchestrator, 2026-10-05), adopting bridle specs with vendored vitest-bridle (mu-943b, mu-hzf9). Local log: meta-notes-ui docs/tickets/open (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.943Z
submitted by external:orchestrator@nuc
