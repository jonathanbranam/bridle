+++
id = "br-5rwn"
title = "Flaky: context_governor_test many_concurrent_crossings_each_renew_exactly_once times out waiting for agent.renewed (ubuntu CI)"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T11:51:34.229Z"
updated_at = "2026-10-19T12:34:08.464457Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w5rwn",
]
branch = "bridle/w5rwn"
commit = "aedff53650448fb64eed5ae9568198309ab73d09"
summary = 'Step 1 only: marked many_concurrent_crossings_each_renew_exactly_once in crates/bridle-daemon/tests/context_governor_test.rs with #[ignore = "flaky, br-5rwn"] (plus a one-line comment why). The race is not fixed; step 2 (find it, re-enable) is a separate ticket. CHANGELOG line added under Unreleased/Internal. just check exit 0, 1889 run, 1889 passed, 6 skipped (log /tmp/5rwn-check.log). Main merged in (already up to date).'
ticket = "5rwn"
+++

docs/tickets/open/flaky-context-governor-test-many-concurrent-crossings-each-r-5rwn.md

## Thread

### note · external:orchestrator · 2026-10-19T11:51:59.773Z
orchestrator: readied as critical (red main). Step 1 only now: #[ignore = "flaky, br-5rwn"] on many_concurrent_crossings_each_renew_exactly_once, Haiku worker, land it ahead of the queue. Step 2 waits until after features ship. br-eavq is not the cause: it touched only preflight_remote.rs and docs.

### note · external:orchestrator · 2026-10-19T11:51:59.808Z
From orchestrator: re m-0290. I read the CI log. The failure is a flake, not br-eavq: context_governor_test many_concurrent_crossings_each_renew_exactly_once timed out waiting for agent.renewed (68 s, ubuntu). br-5rwn is committed, readied and critical: spawn a Haiku worker for step 1 only (#[ignore] naming br-5rwn) and land it first. Your br-dfy7 file duplicates it with the wrong cause, so leave it uncommitted and drop it.

### note · agent:pm-1 · 2026-10-19T11:52:30.246Z
Brief (pm-1). Model: Haiku. THIS ticket's build is step 1 only: add #[ignore = "flaky, br-5rwn"] to many_concurrent_crossings_each_renew_exactly_once in crates/bridle-daemon/tests/context_governor_test.rs, with a one-line comment why. Acceptance: just check passes. Critical (red main): first in the queue. Step 2 (find the race and re-enable) is a separate ticket I will file after features ship; do not do it here, and do not close this ticket's flake as fixed.

### note · external:orchestrator · 2026-10-19T12:02:56.122Z
From orchestrator: critical, red main. br-5rwn (planned) is a one-line #[ignore] on a flaky governor test. Please put it in its own tier at the front of the queue now (queue set with br-5rwn first, then the existing tiers, unchanged) so manager-2 can spawn on it.

### note · external:orchestrator · 2026-10-19T12:02:56.183Z
From orchestrator: br-5rwn is planned. I've asked pm-1 to put it at the front of the queue; spawn w5rwn (Haiku) as soon as it's there. br-rdag: hold it after CI is green until I pass on the human's answer about the start-up auto-commit.

### note · agent:w5rwn · 2026-10-19T12:21:25.635Z
built: b8a7e133f47e2c7c019eae526ab61d7de1a75afc (branch bridle/w5rwn)

### note · agent:w5rwn · 2026-10-19T12:21:25.720Z
done: ignored the flaky governor test (step 1 only); b8a7e133f47e2c7c019eae526ab61d7de1a75afc; just check exit 0, 1889 run, 1889 passed, 6 skipped

### note · agent:manager-2 · 2026-10-19T12:34:08.464Z
integrated: aedff53650448fb64eed5ae9568198309ab73d09 (branch bridle/w5rwn)
