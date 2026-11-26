+++
id = "br-vrrc"
title = "Agents can't submit tickets to another project: forward ticket submit through the agent's own daemon"
kind = "fix"
state = "integrated"
created_at = "2026-11-26T17:12:49.983Z"
updated_at = "2026-11-26T18:08:21.902884Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wvrrc",
]
branch = "bridle/wvrrc"
commit = "d4d260443c1c49cff5e7443403ba0581653a2d4b"
summary = """
`bridle ticket submit --project <other>` works for agents again. When the direct route fails and $BRIDLE_URL/$BRIDLE_TOKEN point at another project's daemon, the CLI hands the submission to its own daemon (POST /v1/submit/remote), which files it on the target with its peer token (POST /v1/forward/submit/ticket, peer token only). Callers with their own way to the target (human, credentialed sessions) go direct as before. The task records `submitted by <principal>@<sender project>` with creator `remote:<principal>@<sender project>`, so task updates and a drop's reason return through the target's outbox.

Decisions and caveats: not queued (the caller needs the task id), so unreachable/unpaired/unknown is an error and nothing is filed anywhere; a lost answer after a timeout may leave the task filed, and the error says so. The target refuses a submission if it has no peer token back to the sender (`bridle token pair` makes both). Who may submit is who may send cross-project mail today (any principal but a peer); not widened to anything else. An older target daemon answers "doesn't have this command; upgrade it" (not covered by a test, relies on the existing client 404 handling). One behaviour change in an existing br-5akm test: an agent with no credential for the target now forwards instead of failing with "$BRIDLE_TOKEN is that daemon's". No migration. Docs: agent-host principals.md, api.md, daemon.md, cli.md. Only submit is forwarded."""
ticket = "vrrc"
+++

docs/tickets/open/agent-submit-forward-vrrc.md

## Thread

### note · external:orchestrator · 2026-11-26T17:13:08.800Z
readied by external:orchestrator

### note · agent:wvrrc · 2026-11-26T17:55:02.717Z
built: 14867506f362c724ecea81fdb67ac4f1964d79b6 (branch bridle/wvrrc)

### note · agent:wvrrc · 2026-11-26T17:55:02.764Z
done: br-vrrc agent ticket submit --project is forwarded by the own daemon; commit 14867506f362c724ecea81fdb67ac4f1964d79b6; env -i just check exit 0, 2042 tests run, 2042 passed, 6 skipped; main already merged; no UI change; the older-target error is not covered by a test

### note · agent:manager-2 · 2026-11-26T18:08:06.431Z
integrated: d4d260443c1c49cff5e7443403ba0581653a2d4b (branch bridle/wvrrc)

### note · agent:manager-2 · 2026-11-26T18:08:06.612Z
ticket file moved to docs/tickets/resolved/agent-submit-forward-vrrc.md

### note · agent:manager-2 · 2026-11-26T18:08:21.902Z
cleanup: removed agent wvrrc, origin branch bridle/wvrrc, branch bridle/wvrrc
