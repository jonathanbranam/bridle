+++
id = "br-ef3u"
title = "A drain waits forever when the daemon's tracking of agents goes stale: find and name what really holds the quiet point"
kind = "bug"
state = "planned"
created_at = "2026-10-12T14:18:25.126Z"
updated_at = "2026-10-17T02:07:33.004733Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "ef3u"
+++

docs/tickets/open/a-drain-waits-forever-when-the-daemon-s-tracking-of-agents-g-ef3u.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:06:32.041Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:25.195Z
Brief (pm-1): step 1 only, an investigation plus the smallest useful fix; no larger design (the ticket says the design needs the human's review first). Model: Sonnet. Read daemon.md 'Restart in place' and the br-y455/br-btdn threads, then crates/bridle-daemon drain_and_restart and how it decides quiet (agent mid-turn, spawn in flight, postpone, landing registry).
Do: (1) write on this thread a list of the remaining ways the daemon's tracking of agents can go stale (agent exited unrecorded, background workers, spawns, turns that never end) with the code path for each; (2) make a drain that is waiting say what holds it BY NAME in the status line and the upgrade_draining wake text (agent id/name and how long its turn has run; spawn id and age; land id; postpone), so a stale one is visible; (3) if a holder is a row with no live process (agent process gone, pid dead), say that in the same line ("stale?") but do not kill or release anything. Tests for the naming with fake holders. Docs: daemon.md, CHANGELOG. Acceptance: just check. Out of scope: automatically releasing stale holders, the manager-waiting-on-just-check idle issue (separate ticket), project-move drain (kt25).
