+++
id = "br-w3pk"
title = "Retire scripts/claude-orchestrator and claude-advisor: the orchestrator supervisor still relaunches through the wrapper"
kind = "chore"
state = "planned"
created_at = "2026-10-06T23:08:51.212Z"
updated_at = "2026-10-07T03:51:33.135609Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
summary = """Step one of two. [orchestrator] launcher is now Option<String>, unset by default; the supervisor types `bridle session orchestrator --project <project>` (project = the daemon's project name, shell-quoted if needed) into the pane, or a configured launcher verbatim (no repo-path resolution). New orchestrator::launch_line does this; lib.rs calls it. The exact old default string "scripts/claude-orchestrator" in a config is treated as unset (config.rs merge). The pane's shell finds bridle on its PATH, as the advisor relaunch (session.rs relaunch_command) already assumes; nothing else resolves it. Docs (orchestrator-supervision.md, roles-and-config.md) and CHANGELOG updated. Scripts and other mentions untouched (step two)."""
+++

original id: w3pk
Ticket (the human's words; read all of it): docs/tickets/open/retire-scripts-claude-orchestrator-and-claude-advisor-the-or-w3pk.md . THIS TASK IS STEP ONE OF TWO: change the relaunch line only. Do NOT delete scripts/claude-orchestrator or scripts/claude-advisor and do not remove their mentions in cli.md, session.rs, commands/orchestrator.rs or bridle-mail/src/local.rs: step two (a separate task) does that after the daemon has upgraded, because the running daemon types the script's path in the clone and deleting it in the same merge would break the orchestrator relaunch.
Goal: the supervisor relaunches the orchestrator by typing `bridle session orchestrator --project <project>` into the tagged pane, not a repo script path.
EXACT BEHAVIOUR:
- `[orchestrator] launcher` (crates/bridle-daemon/src/config.rs, OrchestratorConfig): the default changes from "scripts/claude-orchestrator" to unset (Option<String>, None). Keep the key as an override.
- crates/bridle-daemon/src/orchestrator.rs: when `launcher` is None, type exactly `bridle session orchestrator --project <project>` (the project name the daemon serves; use the same source the rest of the supervisor and `bridle session` use for it) into the pane on every relaunch. When `launcher` is Some(s), type s verbatim and do NOT resolve it as a repo path (drop the absolute-path resolution). Check how the pane's shell finds `bridle` (PATH): follow what session.rs / the advisor relaunch already assumes, and say in your done note what you found.
- Backward compatibility: an existing config that sets launcher = "scripts/claude-orchestrator" still works because verbatim typing in the repo root... if it does not (the pane's cwd is not the repo root), treat the exact old default string "scripts/claude-orchestrator" as unset. No config sets it today (~/.bridle/config.toml and .bridle/config.toml), so this is only a guard; say what you did.
- Docs, update in the same change: docs/design/agent-host/orchestrator-supervision.md (the relaunch line), docs/design/agent-host/roles-and-config.md (the `launcher` default and meaning), CHANGELOG.md entry (read it with a limit; entries go on top). Code comments in crates/bridle-daemon/src/orchestrator.rs that name the script.
Migration: none (the default changes, no stored state; the daemon picks it up on its next upgrade).
Acceptance: just check passes; the supervisor's tests with the fake tmux show `bridle session orchestrator --project <project>` typed into the pane on relaunch, and a custom `launcher` typed verbatim.
Model: Sonnet (it touches the relaunch path). Out of scope: deleting the scripts and the other doc/comment mentions (step two, br-ynfz placeholder: see the task thread), any change to `bridle session`.

## Thread

### note · external:orchestrator · 2026-10-06T23:09:22.000Z
From orchestrator: br-w3pk is ready, a small cleanup the human raised. The daemon still relaunches the orchestrator through scripts/claude-orchestrator. It goes in two steps: change the relaunch line first, delete the scripts after the daemon upgrades. The brief is the ticket. Place it as normal work, not urgent.

### note · agent:pm-1 · 2026-10-06T23:09:46.340Z
pm-1: in the brief, the 'br-ynfz placeholder' for step two is br-f3sz (pending; it has an edge on this task and a precondition that the daemon runs a build with this change). Plan it once it is open, with dep br-w3pk.

### note · agent:relaunch-w3pk · 2026-10-07T03:51:31.326Z
done: supervisor relaunches via 'bridle session orchestrator --project <project>'; just check exit 0, 1285 tests; 4bf85662

### note · agent:relaunch-w3pk · 2026-10-07T03:51:33.135Z
done: relaunch now types 'bridle session orchestrator --project <project>'; launcher unset by default, custom value typed verbatim; old default string 'scripts/claude-orchestrator' in a config treated as unset (no config sets it). The pane's shell finds bridle on PATH, as the advisor relaunch in session.rs already assumes. just check exit 0, 1285 tests run (1285 passed, 5 skipped), commit 4bf85662. Scripts not deleted.
