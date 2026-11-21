---
id: br-r8td
title: "Per-project startup context: notes aide gets meta-notes prime output injected, no agent choice"
kind: feature
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

For a chosen project and role (first use: the notes project's aide), the output of a project command (`meta-notes prime`, which holds the project's conventions) is put into the session's startup context automatically, in full, with no step the agent runs or can shorten.

The human, 2026-11-21 ~2:00 PM ET, verbatim (via aide@vesta):

"The pre-tool use hook is radical overkill for this problem. This just needs to be enforced without the agent having any control over this. I wasn't realizing that this is not happening in the startup for the aide. This is specifically for the aide at notes, not for any other agent at all. In your instructions, or somehow when you start, that should just be inserted automatically, no matter what, at startup, or just executed automatically, or I'm not sure. The agent should not have any agency over this, but a pre-tool use hook is complete overkill. I don't think that's the right approach at all. Bridle should have some way of just pasting that in for you or forcing you to run it, or something."

## Why (the incident)

The notes aide's startup says to run `meta-notes prime` and follow it (notes `CLAUDE.md`, `.bridle/roles/aide.md`). On 2026-11-21 the aide ran `meta-notes prime 2>&1 | head -80` chained with other commands before seeing the output, missed the conventions (task editing, completion dates), misreported correct behaviour as a bug (orchestrator m-0398, withdrawn m-0402) and hand-edited a task line. Another agent has truncated it the same way before.

## Design guidance (pm-1)

Not a PreToolUse hook (the human rejected it). Preferred shape, smallest first:
- A per-project config setting (in `.bridle/config.toml`, per role) naming a command; the daemon (or `bridle prime <role>`) runs it at session start and appends its full output to the role's startup text, which the session already receives. Look at how `bridle prime aide` / `prime_role_file` (`crates/bridle/src/commands/orchestrator.rs` ~lines 135-165) and the session start path build the startup text, and at how the project's role append works (`docs/design/agent-host/roles-and-config.md`, `docs/design/workflow-layers.md`). Pick the place where the output is already injected without the agent's action.
- The command runs in the project's workspace with a timeout; a failure or empty output is written into the startup text as a visible line ("startup command X failed: ..."), never silently dropped; output is not truncated (cap very large output with a stated limit rather than cutting silently).
- Opt-in per project and role, default off, so no existing project changes. Say in the doc that the command is arbitrary code from the project's own config (same trust as the project's other config) and runs as the daemon user.
- Migration: none needed (new optional key, default off). The notes project then adds one config line (a follow-up for the notes aide/manager, not this ticket).

## Files and docs

`crates/bridle-daemon/src/config.rs` (the setting), the session start / prime path in `crates/bridle/src/commands/orchestrator.rs` and the daemon's startup text builder, `docs/design/agent-host/roles-and-config.md`, `docs/design/cli.md` if `prime` output changes.

## Acceptance

`just check` passes. Tests: a configured command's full multi-hundred-line output appears in the startup text of that role only; another role and a project without the setting are unchanged; a failing command shows a visible failure line. State in the done message how the aide's startup text is delivered (so the worker has verified it reaches the model's context, not only the CLI).

## Model

Sonnet. Out of scope: adding the setting to the notes project's config, a PreToolUse hook, output filtering.
