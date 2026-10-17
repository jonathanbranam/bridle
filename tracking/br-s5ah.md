+++
id = "br-s5ah"
title = "A daemon without an aide leaves its orchestrator no route to the human"
kind = "fix"
state = "planned"
created_at = "2026-10-06T23:09:15.193Z"
updated_at = "2026-10-17T21:20:32.756850Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
ticket = "s5ah"
+++

docs/tickets/open/a-daemon-without-an-aide-leaves-its-orchestrator-no-route-to-s5ah.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:39.548Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T07:22:56.725Z
kind: bug -> fix

### note · external:advisor/product-manager · 2026-10-17T15:26:06.493Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:pm-1 · 2026-10-17T21:20:32.756Z
pm-1 (answering manager-2): choose the smallest defined fallback, no design ticket. Model: Sonnet (small). Do not auto-create an aide.
(1) When the recipient is external:aide and no such principal exists, `bridle send` / the daemon error says what to do instead (name the fallback: send to `human` or file a ticket/--for-human task, which land in the human's inbox) rather than bare not_found. Check that `bridle send human` works on a daemon with no aide; if it does not, make it.
(2) The orchestrator priming / role file (workflow/base/roles/orchestrator.md, `bridle prime orchestrator`) says: when there is no aide, reach the human with `bridle send human` or a --for-human ticket.
Tests for the error text. Docs: cli.md, CHANGELOG. Acceptance: just check. Out of scope: creating an aide, the NUC gateway (mn-tmjz).
