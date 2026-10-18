+++
id = "br-t95y"
title = "Flaky on ubuntu CI: gateway_test a_replaced_binary_is_re_executed exits 1 instead of re-executing"
kind = "fix"
state = "built"
created_at = "2026-10-14T23:43:08.616Z"
updated_at = "2026-10-18T00:09:32.743459Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wt95y",
]
branch = "bridle/wt95y"
commit = "ad3ce8231d2514c216da94d0449ef3c5e7b6eb59"
summary = "The test was not ignored, so I went straight to diagnostics. a_replaced_binary_is_re_executed now sends the gateway's stderr to a file, panics with it if the child exits before listening, and includes it in the final status assertion. Looped 20x on macOS (all passed; cannot reproduce here, Linux not tried), so the cause is still unknown: the next ubuntu CI failure will show it. The port race is not removable from the test side (the gateway binds the port itself). No CHANGELOG entry (test-only). Ticket should stay open until CI shows the cause or stays green."
ticket = "t95y"
+++

docs/tickets/open/flaky-on-ubuntu-ci-gateway-test-a-replaced-binary-is-re-exec-t95y.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T15:26:09.256Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wt95y · 2026-10-18T00:09:32.558Z
built: ad3ce8231d2514c216da94d0449ef3c5e7b6eb59 (branch bridle/wt95y)

### note · agent:wt95y · 2026-10-18T00:09:32.743Z
done: test was not ignored; stderr now captured and early exit reported, 20/20 loops pass on macOS (Linux not reproduced, cause still unknown, so keep an eye on CI); just check exit 0, 1834 tests; ad3ce8231d2514c216da94d0449ef3c5e7b6eb59
