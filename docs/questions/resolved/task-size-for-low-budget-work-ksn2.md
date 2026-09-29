---
id: ksn2
title: An estimated size on tasks, to pick small ones when budget runs short
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [144d8af]
specs: []
needs: []
see: [yurx, 8ups, m7wn, 6t29, j479]
---

## The ask

The human, verbatim (2026-09-28), on the two TUI scroll bugs (yurx, 8ups): "yes, those
could be fixed together. Again, not urgent, but something to do when we're low on budget
because they should be small. We should have an estimated size on these tickets to
indicate small tickets that can be picked up when we're running short on budget."

## Notes

- Tasks have no size today. The queue (j479) is PM-owned tiers; the manager takes from the
  highest tier with a startable task.
- Related: [[maintenance-during-budget-holds-m7wn|maintenance during budget holds]],
  [[budget-presets-and-max-workers-6t29|budget presets and max_workers]].
- First users: [[tui-agents-panel-doesnt-scroll-yurx|yurx]] and
  [[tui-inbox-doesnt-scroll-8ups|8ups]], small and to be fixed together.

## Resolution

Resolved by 144d8af: optional S/M/L task size (`bridle task new|edit --size`).
