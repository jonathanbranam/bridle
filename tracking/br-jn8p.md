+++
id = "br-jn8p"
title = "Orchestrator relaunch starts the session in the pane's old folder, not the home project's repo"
kind = "bug"
state = "built"
created_at = "2026-10-18T13:53:44.516Z"
updated_at = "2026-10-18T20:45:23.218120Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wjn8p",
]
branch = "bridle/wjn8p"
commit = "c8a3ce1e"
summary = "Default orchestrator launch line is now 'cd <repo> && bridle session orchestrator --project <p>' (launch_line in orchestrator.rs takes the repo; lib.rs passes ws.repo), so the session runs in the project's repo whatever the pane's folder. A configured launcher stays verbatim. Unit tests assert the typed and direct-respawn lines; no test runs a real pane. Docs: orchestrator-supervision.md, roles-and-config.md; CHANGELOG under Fixed."
ticket = "jn8p"
+++

docs/tickets/open/orchestrator-relaunch-starts-the-session-in-the-pane-s-old-f-jn8p.md

## Thread

### note · agent:pm-1 · 2026-10-18T13:54:30.487Z
Brief (pm-1): fix the orchestrator relaunch path in crates/bridle-daemon (grep respawn-pane / launcher / "bridle session orchestrator") so the launch line starts in the supervising project's repo, e.g. respawn-pane -c <repo> or a `cd <repo> &&` prefix on the default launcher; a configured [orchestrator] launcher stays verbatim. Check: a test where the pane cwd is another repo yields a session cwd of this project's repo; `just check` passes; update docs/design/agent-host/ if the launcher is described there. Model: Sonnet. Out of scope: other br-7ytg items. Do not touch the 7ytg warning code in the same pass if it conflicts; run after br-7ytg if both edit the same files.

### note · external:aide · 2026-10-18T14:54:40.079Z
The human, 2026-10-18 ~11:00 AM ET (to aide), verbatim: "for br-jn8p I'm fine with B unless we hit an issue". B = no launcher workaround in dalek-home's config; wait for this fix. The orchestrator keeps running from /Volumes/Data/work/bridle/bridle until it lands.

### note · agent:wjn8p · 2026-10-18T20:45:21.075Z
built: c8a3ce1e (branch bridle/wjn8p)

### note · agent:wjn8p · 2026-10-18T20:45:23.218Z
done: default orchestrator launch line is now 'cd <repo> && bridle session orchestrator --project <p>' (custom launcher verbatim); just check exit 0, 1879 tests passed, commit c8a3ce1e. Tests assert the line only (typed and direct respawn); none runs a real pane.
