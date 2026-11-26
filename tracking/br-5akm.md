+++
id = "br-5akm"
title = "ticket submit --project files on the agent's own daemon when BRIDLE_URL is set"
kind = "bug"
state = "built"
created_at = "2026-11-26T16:31:36.042Z"
updated_at = "2026-11-26T17:12:09.475751Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w5akm",
]
branch = "bridle/w5akm"
commit = "dcc65a4a9a3075d66f248b5b227439a0c728b8ee"
summary = """
A typed `--project` now beats an inherited `$BRIDLE_URL` (new crates/bridle/src/commands/target.rs). The URL is kept only if it is that project's daemon; otherwise the command goes to the named project's daemon with `--token`, `$BRIDLE_AS`'s credential or (outside Claude Code) the human token, never `$BRIDLE_TOKEN`; else it fails saying why and sends nothing. `$BRIDLE_PROJECT` alone is unchanged. Docs: daemon.md, principals.md, cli.md. Test: a_typed_project_beats_an_inherited_bridle_url (two throwaway daemons).

Affected: every command connecting through client_for / client_for_read (ticket, task, agent, status, budget, token, schedule, tui, `daemon stop|restart` and so on; `daemon stop --project B` under BRIDLE_URL used to stop the agent's own daemon). Unchanged: send and task watch (own outbox / direct credential), statusline, mail run (already errors). Not touched: the human-only `token pair` verify helper still resolves with BRIDLE_URL first if it is set.

Caveat for follow-up: an agent session has no registry and no token for other daemons, so `ticket submit --project <other>` from an agent now errors and cannot file there. The rule file-tickets-on-other-projects expects it to work; that needs something like submission forwarding through the agent's own daemon (as mail does)."""
ticket = "5akm"
+++

docs/tickets/open/submit-project-url-5akm.md

## Thread

### note · external:orchestrator · 2026-11-26T16:31:40.609Z
readied by external:orchestrator

### note · agent:pm-1 · 2026-11-26T16:32:15.548Z
pm-1 brief: model Sonnet. Look at daemon discovery order (docs/design/agent-host/daemon.md) and how crates/bridle/src picks URL and token for --project; an explicit --project flag must beat an inherited BRIDLE_URL (and pick the matching token, never sending this agent's token to another daemon without checking principals.md), or error saying why. BRIDLE_PROJECT env alone should not change today's behaviour unless the docs say so. Check other commands that share this resolution and say which are affected. Update daemon.md in step. just check passes; no migration.

### note · agent:w5akm · 2026-11-26T17:11:22.959Z
handoff: fix and test committed as dcc65a4a on bridle/w5akm (already up to date with main). just check is running in the background; claim is kept until it passes, then push, summary, built.

### note · agent:w5akm · 2026-11-26T17:12:09.412Z
built: dcc65a4a9a3075d66f248b5b227439a0c728b8ee (branch bridle/w5akm)

### note · agent:w5akm · 2026-11-26T17:12:09.475Z
done: br-5akm fixed: a typed --project beats BRIDLE_URL or fails saying why; commit dcc65a4a9a3075d66f248b5b227439a0c728b8ee; just check (env -i) exit 0, 2034 tests run, 2034 passed, 6 skipped. No UI change. Note: agents still cannot file on OTHER projects (no registry or token), they now get an error instead; rule file-tickets-on-other-projects needs submit forwarding through the own daemon, worth a follow-up ticket. Details in the ticket summary.
