+++
id = "br-mt4v"
title = "Tests for eggc: one incident per land past the cap; a land refused during a drain runs after the restart; check nextest children survive the cap"
kind = "fix"
state = "claimed"
created_at = "2026-10-16T10:03:26.182Z"
updated_at = "2026-10-16T11:35:40.858770Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:mt4v",
]
summary = """Tests for eggc: restart_land_test.rs (a land past the cap gives exactly one restart.land_forced event and one incident and its check is stopped; a land refused while the lands are closed gets 503 draining and runs once reopened, which stands for the restart since a new daemon's Lands starts open) and a server.rs unit test for a check child in its own process group. The child DID survive killpg (POSIX: setpgrp children leave the group), so fixed: OverdueLand::stop_check snapshots the check's descendants via ps and SIGKILLs them with the group. Test seams: Lands::set_cap and RunningDaemon.lands, close_if_quiet made pub. Caveat: the drain loop breaks after the first forced pass, so "one incident" holds per restart attempt; if the restart then fails after forcing, a retried drain would report a still-running land again."""
ticket = "mt4v"
+++

docs/tickets/open/tests-for-eggc-one-incident-per-land-past-the-cap-a-land-ref-mt4v.md
