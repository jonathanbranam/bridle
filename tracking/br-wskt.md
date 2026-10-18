+++
id = "br-wskt"
title = "manager-2 goes idle with built work and free slots: it misses 'built' notes and doesn't refill slots"
kind = "bug"
state = "built"
created_at = "2026-10-18T03:10:10.581Z"
updated_at = "2026-10-18T07:07:55.068008Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wwskt",
]
branch = "bridle/wwskt"
commit = "9bb1a93cc78e58c113287b9f8bbd4014cae47cb0"
summary = "A drain whose restart is postponed (bridle daemon restart --postpone) no longer holds message delivery: AgentManager::holds_turns() (draining and not postponed) replaces draining() at the two delivery sites in supervisor.rs, and postpone_until delivers messages held before the postpone. Spawns/claims stay refused. Test in restart_hold_test.rs; daemon.md and CHANGELOG updated. This is the orchestrator's found cause only; the broader manager-idles/slot-refill questions on the ticket are not addressed."
ticket = "wskt"
+++

docs/tickets/open/manager-2-goes-idle-with-built-work-and-free-slots-it-misses-wskt.md

## Thread

### note · external:orchestrator · 2026-10-18T03:11:08.939Z
orchestrator, 03:12Z: likely part of the cause. The daemon is draining for self-upgrade 6457a600, postponed until 03:32Z (manager-2 itself ran 'bridle daemon restart --postpone 45m' before landing ussh). While draining, messages to agents stay 'arrived', never delivered: my m-16241 and m-16245 to manager-2 sit undelivered and manager-2 has had no turn since 03:09Z. So a manager that postpones an upgrade to finish a land then cannot hear 'built' notes or nudges for the rest of the postponement. Compare the 2026-10-12 01:56Z incident (br-up5r). Fix direction: a postponed drain should not hold message delivery to idle agents, or a land's postpone should end when the land does.

### note · agent:wwskt · 2026-10-18T07:07:54.865Z
built: 9bb1a93cc78e58c113287b9f8bbd4014cae47cb0 (branch bridle/wwskt)

### note · agent:wwskt · 2026-10-18T07:07:55.068Z
done: postponed drain no longer holds message delivery (holds_turns, postpone delivers held); just check exit 0, 1862 tests passed; 9bb1a93cc78e58c113287b9f8bbd4014cae47cb0
