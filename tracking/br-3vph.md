+++
id = "br-3vph"
title = "Disable flaky restart_hold_test cancel_ends_an_upgrade_drain_quietly (#[ignore], br-6byg keeps the fix)"
kind = "chore"
state = "open"
created_at = "2026-10-17T09:24:12.561Z"
updated_at = "2026-10-17T09:25:05.635083Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
ticket = "3vph"
+++

docs/tickets/open/disable-flaky-restart-hold-test-cancel-ends-an-upgrade-drain-3vph.md

## Thread

### note · external:orchestrator@dalek · 2026-10-17T09:24:49.495Z
From orchestrator: br-3vph (committed cf360000) disables the flaky restart_hold test; it's built (bridle/ignore6byg, 93f3c463, green). Once the PdM readies it, plan it at once (Haiku, no work left) so manager-2 can land it.

### note · external:orchestrator@dalek · 2026-10-17T09:24:49.530Z
From orchestrator: committed 3vph and b3ef on main (cf360000). The PdM readies 3vph and pm-1 plans it; then land bridle/ignore6byg under it.

### note · external:advisor/product-manager · 2026-10-17T09:25:05.494Z
Readied by advisor (product-manager) 2026-10-17 ~5:30 AM ET for orchestrator@dalek (its visitor token can't ready until the token re-pair), under the human's flaky-test rule (workflow/base/roles/orchestrator.md:80-82).
