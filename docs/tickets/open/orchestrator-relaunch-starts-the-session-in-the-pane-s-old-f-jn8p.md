---
id: br-jn8p
title: Orchestrator relaunch starts the session in the pane's old folder, not the home project's repo
kind: bug
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-uysm, br-7ytg, br-8uxe]
---

## The ask

The human, 2026-10-18 ~10 AM ET (via aide): "It's in the wrong folder though. I can't move it since the daemon restarts it so fast".

After br-uysm, dalek-home's supervisor relaunched the orchestrator correctly as `bridle session orchestrator --project dalek-home` (13:51:43Z), but in pane %4's shell, whose folder is still /Volumes/Data/work/bridle/bridle (the pane was first opened for bridle). `bridle session` takes its repo from the cwd, so Claude Code runs in bridle's repo (its CLAUDE.md, settings, trust) while serving dalek-home. The human can't cd first: the supervisor relaunches within seconds of an /exit.

Ask: the relaunch starts the session in the supervising project's repo whatever the pane's folder (e.g. `respawn-pane -c <repo>`, or the default launcher line is `cd <repo> && bridle session orchestrator --project <p>`). Test: a pane whose cwd is another repo gets a session whose cwd is this project's repo. Workaround until then: `[orchestrator] launcher = "cd <repo> && bridle session orchestrator --project <p>"` in the home's config (typed verbatim). Related: br-7ytg item C.
