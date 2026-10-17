+++
id = "br-6byg"
title = "Flaky: restart_hold_test cancel_ends_an_upgrade_drain_quietly hit its 60 s hang guard on CI"
kind = "bug"
state = "open"
created_at = "2026-10-17T01:43:51.292Z"
updated_at = "2026-10-17T15:26:10.170282Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "6byg"
+++

docs/tickets/open/flaky-restart-hold-test-cancel-ends-an-upgrade-drain-quietly-6byg.md

## Thread

### note · external:orchestrator@dalek · 2026-10-17T08:55:54.642Z
From orchestrator: under the human's flaky-test rule (2026-10-11: 'disable it and ... file a ticket ... send to the PdM'), disable restart_hold_test cancel_ends_an_upgrade_drain_quietly now: #[ignore = "flaky, br-6byg"], nothing else. Do it as a tiny Haiku worker or within your next land, whichever your workflow allows, ahead of 9qws. The real fix stays on br-6byg, which I'm sending to the PdM to schedule after features. Stop retrying lands for it once it's disabled.

### note · agent:ignore6byg · 2026-10-17T09:23:53.114Z
done: cancel_ends_an_upgrade_drain_quietly is #[ignore = "flaky, br-6byg"] (test body unchanged); CHANGELOG Unreleased line added (test changes are listed there, e.g. k562); just check exit=0, 1802 run, 1802 passed, 7 skipped (last full landing 1809, in band); main already merged (no-op); commit 93f3c4639ff96c1f7b1686df8d49bd77d82d514f

### note · agent:ignore6byg · 2026-10-17T09:31:39.332Z
done: merged main (5febee5f, br-4cgx) into bridle/ignore6byg; just check exit=0 on the merge, 1808 run, 1808 passed, 7 skipped; tip 3e1c... see sha

### note · agent:ignore6byg · 2026-10-17T09:31:41.613Z
correction to m-15394: the tip is def68f77ee22f86ab9fabdeaa6400f1ba311143d (merge of main 5febee5f into bridle/ignore6byg, on top of 93f3c463). just check ran on that tip: exit 0, 1808 run, 1808 passed, 7 skipped. Ignore the placeholder sha in m-15394.

### note · external:advisor/product-manager · 2026-10-17T15:26:09.933Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.
