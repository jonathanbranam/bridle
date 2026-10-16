+++
id = "br-rnhp"
title = "bridle up starts the orchestrator in no project: it should start it in <host>-home"
kind = "bug"
state = "integrated"
created_at = "2026-10-16T01:53:42.900Z"
updated_at = "2026-10-16T15:37:18.086832Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:rnhp",
]
branch = "bridle/rnhp"
commit = "0e87928104f3d2fdd4a83d00df423522e6b15fbb"
summary = "`bridle up` now starts the orchestrator with `--project <host>-home` when [machine] name is set and `[projects]` puts `<host>-home` on this machine; otherwise it runs the old command and the output says no home was enrolled and the project comes from $BRIDLE_PROJECT or the cwd. Pure fn `orchestrator_command` in crates/bridle/src/up.rs, with a test (enrolled, other machine's home, no name, no config). No flag or unit change: the boot unit runs `bridle up`, which reads the config at run time. Docs: add-a-machine.md step 7 (the issue said step 8), cli.md, CHANGELOG. just check green: 1767 passed, commit cedc5e4f."
ticket = "rnhp"
+++

docs/tickets/open/bridle-up-starts-the-orchestrator-in-no-project-it-should-st-rnhp.md

## Thread

### note · external:aide · 2026-10-16T14:51:49.244Z
Readied on the human's approval, via aide (2026-10-16 ~9:30 AM ET), verbatim: "Approve rnhp and bft2."

### note · agent:pm-1 · 2026-10-16T14:52:29.842Z
Brief (pm-1): the ask is the brief. Model: Sonnet (small). Files: crates/bridle/src/up.rs, docs/context/add-a-machine.md step 8, cli.md if `up` flags change. Acceptance: just check, with a test that the command `up` builds names <host>-home when enrolled and the fallback says which project it used. Out of scope: boot sessions, startup cap/stagger, orchestrator starting aides (held for br-mru4).

### note · agent:rnhp · 2026-10-16T15:19:08.912Z
built: cedc5e4f (branch bridle/rnhp)

### note · agent:rnhp · 2026-10-16T15:19:08.985Z
done: bridle up starts the orchestrator in <host>-home when enrolled (check exit 0, 1767 passed); cedc5e4f

### note · agent:rnhp · 2026-10-16T15:19:21.195Z
correction: tip is ded97743 (adds a docs-only cli.md line to cedc5e4f, where the check ran green)

### note · agent:manager-2 · 2026-10-16T15:37:18.086Z
integrated: 0e87928104f3d2fdd4a83d00df423522e6b15fbb (branch bridle/rnhp)
