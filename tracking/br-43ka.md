+++
id = "br-43ka"
title = "Statusline usage from sessions in any project but the token's is dropped silently (orchestrator in dalek-home, aides in other projects)"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T23:11:39.068Z"
updated_at = "2026-10-20T02:06:18.740165Z"
created_by = "external:advisor/usage"
watchers = [
    "external:advisor/usage",
    "agent:w43ka",
]
branch = "bridle/w43ka"
commit = "637b99516ffa097482cb83a76fe9a1ac2a5bb36e"
summary = "post_statusline_report_via (commands/orchestrator.rs): a 401/403 from the session's project daemon retries the report on the other live daemons by name, identity suffixed ' @<project>'; if none accepts it returns false and the statusline appends ' | usage not counted' (plus a stderr warning). Chose retry-with-tag over a machine-wide credential or minting per daemon: no daemon/setup change. Caveat: the usage lands in another project's DB. Docs: cli.md, usage-and-budget.md. Tests: two mock-server tests."
ticket = "43ka"
+++

docs/tickets/open/statusline-usage-from-sessions-in-any-project-but-the-token-43ka.md

## Thread

### note · external:advisor/usage · 2026-10-19T23:11:59.919Z
From advisor (usage): filed from the week's usage analysis and readied (kind bug, rule ready-without-asking).

### note · agent:w43ka · 2026-10-20T02:05:56.014Z
built: 05549c399059e3c5ac43d30152da4b42e111a3f1 (branch bridle/w43ka)

### note · agent:w43ka · 2026-10-20T02:05:56.524Z
done: statusline refused post retried on other daemons with @project tag, 'usage not counted' on the line if none accepts; check exit 0, 1910 tests; 05549c399059e3c5ac43d30152da4b42e111a3f1

### note · agent:manager-2 · 2026-10-20T02:06:03.443Z
integrated: 637b99516ffa097482cb83a76fe9a1ac2a5bb36e (branch bridle/w43ka)

### note · agent:manager-2 · 2026-10-20T02:06:18.740Z
cleanup: removed agent w43ka, branch bridle/w43ka
