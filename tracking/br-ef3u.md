+++
id = "br-ef3u"
title = "A drain waits forever when the daemon's tracking of agents goes stale: find and name what really holds the quiet point"
kind = "bug"
state = "planned"
created_at = "2026-10-12T14:18:25.126Z"
updated_at = "2026-10-17T09:47:17.122473Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:ef3u",
]
branch = "bridle/ef3u"
commit = "62459ad202647d9e23f9e1a4a4b5511200b42330"
summary = 'A waiting drain names its holders (server.rs drain_holder_labels, used by status draining_on and the upgrade_draining wake): agent mid-turn with turn age, spawns, lands with age, postponement. A busy agent whose pid is dead or missing is marked "stale?"; nothing is released. CLI drain line joins with "; ". Unit test with fake holders; upgrade_test updated. The list of ways tracking can go stale is in daemon.md ("What holds a drain"); found by reading the code, the reaper path was not audited in depth. Wake text changed from "still in a turn" to "still held by".'
ticket = "ef3u"
+++

docs/tickets/open/a-drain-waits-forever-when-the-daemon-s-tracking-of-agents-g-ef3u.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:06:32.041Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:25.195Z
Brief (pm-1): step 1 only, an investigation plus the smallest useful fix; no larger design (the ticket says the design needs the human's review first). Model: Sonnet. Read daemon.md 'Restart in place' and the br-y455/br-btdn threads, then crates/bridle-daemon drain_and_restart and how it decides quiet (agent mid-turn, spawn in flight, postpone, landing registry).
Do: (1) write on this thread a list of the remaining ways the daemon's tracking of agents can go stale (agent exited unrecorded, background workers, spawns, turns that never end) with the code path for each; (2) make a drain that is waiting say what holds it BY NAME in the status line and the upgrade_draining wake text (agent id/name and how long its turn has run; spawn id and age; land id; postpone), so a stale one is visible; (3) if a holder is a row with no live process (agent process gone, pid dead), say that in the same line ("stale?") but do not kill or release anything. Tests for the naming with fake holders. Docs: daemon.md, CHANGELOG. Acceptance: just check. Out of scope: automatically releasing stale holders, the manager-waiting-on-just-check idle issue (separate ticket), project-move drain (kt25).

### note · agent:ef3u · 2026-10-17T07:42:22.487Z
A waiting drain names its holders (server.rs drain_holder_labels, used by status draining_on and the upgrade_draining wake): agent mid-turn with turn age, spawns, lands with age, postponement. A busy agent whose pid is dead or missing is marked "stale?"; nothing is released. CLI drain line joins with "; ". Unit test with fake holders; upgrade_test updated. The list of ways tracking can go stale is in daemon.md ("What holds a drain"); found by reading the code, the reaper path was not audited in depth. Wake text changed from "still in a turn" to "still held by".

### note · agent:ef3u · 2026-10-17T07:42:22.532Z
built: 499c7666 (branch bridle/ef3u)

### note · agent:ef3u · 2026-10-17T07:42:22.658Z
done: drain holders named in status + upgrade_draining wake, stale? on dead pid; just check exit 0, 1795 tests passed, sha 499c7666

### note · agent:manager-2 · 2026-10-17T08:02:40.497Z
integrated: 62459ad202647d9e23f9e1a4a4b5511200b42330 (branch bridle/ef3u)

### note · agent:manager-2 · 2026-10-17T08:03:02.195Z
cleanup: removed agent ef3u, branch bridle/ef3u

### note · external:advisor/product-manager · 2026-10-17T09:47:05.675Z
Checked by advisor (product-manager) 2026-10-17 ~5:50 AM ET: merged (62459ad2), code complete. NOT resolving yet, stale docs: daemon.md:353 and cli.md:80 still say 'waiting on <agents mid-turn>' (now all holders, '; ' separated, 'stale?'); daemon.md:365 still says the wake names agents in a turn (now 'still held by:'). pm-1: a small doc follow-up under this ticket, please.

### note · agent:pm-1 · 2026-10-17T09:47:15.601Z
Follow-up brief (pm-1), from the PdM's check. Model: Haiku (docs only). Update three stale doc lines for the built drain-holder naming: docs/design/agent-host/daemon.md ~353 and docs/design/cli.md ~80 still say 'waiting on <agents mid-turn>'; they now list all holders ('; ' separated: agent mid-turn with turn age, spawns, lands, postponement, with 'stale?' on a dead pid). daemon.md ~365 still says the wake names agents in a turn; the wake text is now 'still held by: ...'. Docs only, no code. Acceptance: just check (or the docs checks if that is all that runs). Out of scope: any behaviour change.
