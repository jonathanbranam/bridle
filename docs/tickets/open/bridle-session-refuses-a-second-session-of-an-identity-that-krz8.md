---
id: krz8
title: bridle session refuses a second session of an identity that's already running
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [3397]
tasks: [br-krz8]
---

## The ask

From [[bridle-session-ignores-the-folder-it-runs-in-and-defaults-to-3397|3397]]: running
`bridle session aide` in bridle-ui's folder started a second `external:aide` for bridle while one
was already running. 3397 fixed the project choice but not the duplicate. The human approved this as
a follow-up (2026-10-04, via the aide): "Agree with 2".

`bridle session <role>` refuses to start when a session of the same identity in the same project
is registered and its process is alive (`bridle status` sessions, the pid). The message names the
running one (pid, pane, machine) and how to replace it (`bridle session restart <identity>`). A
registered session whose process is gone doesn't block. Test with a stub claude
(`BRIDLE_LAUNCHER_TEST=1`). Docs: cli.md's `bridle session`.
