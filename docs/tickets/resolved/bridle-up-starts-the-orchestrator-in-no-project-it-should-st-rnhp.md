---
id: br-rnhp
title: "bridle up starts the orchestrator in no project: it should start it in <host>-home"
kind: bug
created: 2026-10-16
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-rjd5, br-4r3k]
resolved: 2026-10-16T21:08:11Z
---

## The ask

Found 2026-10-15 answering vesta's question (aide m-13897): "which project does bridle up's boot
hook start the orchestrator in?"

Read in code: `bridle up` (crates/bridle/src/up.rs `run`) opens a tmux window running
`<exe> session orchestrator` with no `--project` and no `-c <dir>`, and the launchd plist /
systemd unit it installs sets no WorkingDirectory or BRIDLE_PROJECT. `session` resolves the
project from `--project`, `$BRIDLE_PROJECT`, then the cwd's workspace (crates/bridle/src/project.rs
`resolve`). At boot the pane's cwd is the tmux server's (usually $HOME), which is no workspace, so
the session fails with "no project", or starts in whatever project tmux's cwd happens to be in.
So `bridle up` knows nothing of the home project (docs/context/add-a-machine.md section 6).

Wanted: `bridle up` starts the orchestrator with `--project <host>-home` when that project is in
`[projects]` for this machine (host = this machine's name), else falls back to today's
behaviour and says which project it used (or `--project` on `bridle up`, written into the unit by
`--install`). Test: the command `up` builds names the home project when enrolled. Docs:
add-a-machine.md step 8 (the move is then "restart the orchestrator; `bridle up` picks the home").
