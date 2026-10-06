---
id: x56y
title: "bridle send from any agent fails since the 3haz build: a daemon found by BRIDLE_URL has no project, so BRIDLE_PROJECT reads as another project"
kind: bug
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [3haz, 2ax5]
tasks: []
---

## The ask

Found 2026-10-06 ~23:15 UTC (orchestrator, from aide's report that bridle-ui's manager-1 went quiet). Since the daemons restarted onto the br-3haz build (~22:12 UTC), no agent on any daemon can `bridle send`. Seen on manager-1 and fm-links3 (bridle-ui) and on manager-2 (bridle). Reports that workers send to their managers are lost, so managers wait forever (ui-7veu sat landable for an hour).

Errors: `bad_request: that is this daemon's own project: send without --project`, or, with `--task`, `--task isn't supported for another project's daemon yet`.

Cause (from reading the code, and reproduced): every agent runs with `BRIDLE_PROJECT=<its project>` and `BRIDLE_URL`, so clap fills `cli.project`. `own_daemon_for_other_project` (`crates/bridle/src/commands/misc.rs`) resolves the own daemon from `BRIDLE_URL`. A daemon found by URL has `project == None`, so the check `own.project == Some(project)` fails and the send goes to the own daemon's outbox as cross-project mail. The daemon refuses it (`crates/bridle-daemon/src/server.rs`, "own project"). Repro: `BRIDLE_URL=http://127.0.0.1:7401 BRIDLE_PROJECT=bridle bridle send <x> ...`.

Workaround, tested and sent to every manager: `env -u BRIDLE_PROJECT bridle send ...`.

Fix: a send is cross-project only when the target project is known to differ from the own daemon's. When the own daemon was found by URL with no project, learn its project (from the daemon, e.g. its status, or from `BRIDLE_PROJECT` only when that came from the agent env), or treat an unknown own project as "same" and send directly. Also check the other `--project` paths that use `own_daemon_for_other_project`, or the same comparison, for the same mistake.

Check: a CLI test with `BRIDLE_URL` and `BRIDLE_PROJECT` set to the daemon's own project sends directly, with `--task` too; a different project still goes through the outbox. `just check`.

Test gap (as in 2ax5): no test ran `bridle send` with an agent's real environment. Add one. Model: Sonnet. Critical: it breaks every role's messaging on every daemon.
