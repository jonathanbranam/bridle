+++
id = "br-zwup"
title = "Small gaps: stale reopened line in roles-and-lifecycle, mail watchdog outside S3 calls"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T10:03:36.772Z"
updated_at = "2026-10-16T12:34:39.718411Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:zwup",
]
branch = "bridle/zwup"
commit = "04bef1731b0bae2595d35b75c65c2d6c5a2d7a7f"
summary = "Two small fixes. (1) docs/design/roles-and-lifecycle.md: the task plan line said only `open` is accepted and that a reopened task is a conflict; the built behaviour (plan_task, cli.md, api.md) accepts `open` and `reopened`, so the doc now says that. (2) Mail watchdog (crates/bridle-mail/src/bridge.rs): the no-successful-poll check ran between polls, so a hang anywhere inside a poll (a daemon send, the outbound feed, not only S3) never reached it. run() now races the poll loop (poll_forever) against a separate watchdog future (silent_for) fed by a watch channel that each successful inbound poll updates; the exit message is unchanged. Test: the_watchdog_catches_a_hang_after_the_store_read (a sink that never answers, paused time). docs/design/mail.md and CHANGELOG (Unreleased) updated. Caveat: a poll that hangs in the middle is dropped when the watchdog fires; the process exits as before, and the unit restarts it."
ticket = "zwup"
+++

docs/tickets/open/small-gaps-stale-reopened-line-in-roles-and-lifecycle-mail-w-zwup.md

## Thread

### note · agent:zwup · 2026-10-16T12:19:53.480Z
built: de7d33af (branch bridle/zwup)

### note · agent:zwup · 2026-10-16T12:19:55.268Z
done: br-zwup built: watchdog runs beside the poll loop (hang outside S3 caught, new test), roles-and-lifecycle reopened line fixed, mail.md + CHANGELOG updated; just check exit 0, 1761 tests run / 1761 passed / 6 skipped; merge of main was already up to date; commit de7d33af

### note · agent:manager-2 · 2026-10-16T12:34:39.718Z
integrated: 04bef1731b0bae2595d35b75c65c2d6c5a2d7a7f (branch bridle/zwup)
