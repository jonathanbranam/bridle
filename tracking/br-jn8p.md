+++
id = "br-jn8p"
title = "Orchestrator relaunch starts the session in the pane's old folder, not the home project's repo"
kind = "bug"
state = "claimed"
created_at = "2026-10-18T13:53:44.516Z"
updated_at = "2026-10-18T19:05:28.595651Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wjn8p",
]
ticket = "jn8p"
+++

docs/tickets/open/orchestrator-relaunch-starts-the-session-in-the-pane-s-old-f-jn8p.md

## Thread

### note · agent:pm-1 · 2026-10-18T13:54:30.487Z
Brief (pm-1): fix the orchestrator relaunch path in crates/bridle-daemon (grep respawn-pane / launcher / "bridle session orchestrator") so the launch line starts in the supervising project's repo, e.g. respawn-pane -c <repo> or a `cd <repo> &&` prefix on the default launcher; a configured [orchestrator] launcher stays verbatim. Check: a test where the pane cwd is another repo yields a session cwd of this project's repo; `just check` passes; update docs/design/agent-host/ if the launcher is described there. Model: Sonnet. Out of scope: other br-7ytg items. Do not touch the 7ytg warning code in the same pass if it conflicts; run after br-7ytg if both edit the same files.

### note · external:aide · 2026-10-18T14:54:40.079Z
The human, 2026-10-18 ~11:00 AM ET (to aide), verbatim: "for br-jn8p I'm fine with B unless we hit an issue". B = no launcher workaround in dalek-home's config; wait for this fix. The orchestrator keeps running from /Volumes/Data/work/bridle/bridle until it lands.
