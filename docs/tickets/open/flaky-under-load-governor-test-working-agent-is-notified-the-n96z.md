---
id: n96z
title: "Flaky under load: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out"
kind: bug
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-n96z]
---

## The ask

Filed by advisor (product-manager), 2026-10-10 ~12:45 AM ET, from three task threads tonight.

`governor_test working_agent_is_notified_then_stopped_when_its_turn_ends` times out (70 s) when
the machine is loaded, and passes when run alone:

- br-7ufd (w7ufd, 2026-10-09 ~8:50 PM ET): "one earlier governor_test timeout was load, 17/17 on
  rerun".
- br-fpde (wfpde, 2026-10-10 ~12:24 AM ET): full `just check` at load ~24-45 on 16 cores exited
  100, 1057/1058 passed, the only failure this test (70 s timeout); passes alone. Three earlier
  runs failed on load-sensitive tests too. Still not green.
- br-hdbj (whdbj, 2026-10-10 ~12:42 AM ET): green only with `NEXTEST_RETRIES=2` because of this
  test.

Cost: each failed full check costs a worker ~20-40 min of build and test on the shared machine,
and tonight it held up br-fpde, which is in the v0.6.0 release for the Windows PC.

Wanted: the test passes reliably under the load that bridle's own workers create (three workers'
checks at once), without weakening what it checks: make its timing depend on events, not wall
clock, or give it a deadline scaled to the load, as br-6nzj and br-8ff8 did for theirs. Find why
a loaded machine needs more than 70 s first.
