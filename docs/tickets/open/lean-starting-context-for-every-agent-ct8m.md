---
id: ct8m
title: Lean starting context for every agent, the orchestrator included
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [the-orchestrator-stays-running-fx7x, accurate-context-kc4v]
---

## The ask

The human (2026-09-29, to the orchestrator):

> let's make a plan to trim your context and every agent's context, particular background
> agents. Look for tools that we are unlikely to use; remove the google connectors for background
> agents they should have very few tools; don't need askuserquestion for example. There is a Matt
> Pocock video about this maybe a post we can find he trimmed starting context way down.

## What we measured

- The orchestrator (interactive, the human's full settings): about **50K** at its first turn;
  about 9K of that is `bridle prime orchestrator` (the state file is 22.6 KB, mostly history).
  It reached 134K within 48 busy minutes (~1.75K/min).
- Background agents show 48-60K after one turn in `bridle agents`.
- Spike 01 (`docs/spikes/01-stream-json-findings.md:42`): a default headless session carries
  ~6.5K system prompt, ~9.6K tools (plus ~15.9K deferred) and ~2.0K skills.

## What bridle does today (crates/bridle-claude/src/command.rs:107-181, bridle-daemon config.rs:275-415)

- Spawned agents already get `--strict-mcp-config` (no `.mcp.json` or user MCP servers),
  `--setting-sources project` (no user settings, plugins or hooks) and
  `--exclude-dynamic-system-prompt-sections`.
- But tools are only **permission-gated**: `--allowedTools`/`--disallowedTools` with scoped
  rules. No `--tools` is passed, so every built-in tool's definition (Workflow, Monitor,
  DesignSync, Artifact, AskUserQuestion, Cron*, NotebookEdit, plan mode, ...) is still in each
  agent's context, even though it can't be used.
- Unverified: whether `--strict-mcp-config` also blocks the claude.ai connectors (Gmail, Drive,
  Calendar, Claude Docs).
- The orchestrator and advisor scripts pass no trimming flags at all.

## Prior art

Matt Pocock, "How to kill the bloat in Claude Code's system prompt"
(https://www.aihero.dev/how-to-kill-the-bloat-in-claude-codes-system-prompt):
- Measure with `/context`; rank tool sizes with a logging proxy (`ANTHROPIC_BASE_URL`).
- A **bare tool name** in `permissions.deny` removes the definition; a scoped rule like
  `Bash(rm *)` only blocks calls. His deny list: EnterPlanMode, ExitPlanMode, DesignSync,
  NotebookEdit, SendMessage, PushNotification, RemoteTrigger, ReportFindings, ScheduleWakeup,
  AskUserQuestion, CronCreate, CronDelete, CronList.
- Settings: `disableBundledSkills`, `disableWorkflows`, `disableClaudeAiConnectors`,
  `disableArtifact`, `disableRemoteControl`; `skillOverrides` per skill (`off` or
  `user-invocable-only`). Examples: Workflow ~5.3K, DesignSync ~2.2K, Monitor ~1.9K tokens.
- "Tens of thousands of tokens per turn" saved.

## Plan

1. **Spike, done: `docs/spikes/08-lean-context-findings.md`** (`--tools` is the lever; first turn 19K to 12.8K for a worker; strict MCP already blocks the connectors; the premise that every tool definition is loaded is mostly wrong, 11K are deferred). Original ask: **Spike (one worker, cheap live runs on Haiku in a scratch dir).** On the installed Claude
   Code, measure starting context (`get_context_usage`, kc4v) for a worker and a manager spawn:
   today's flags; plus `--tools <whitelist>`; plus bare-name `--disallowedTools`; plus the
   settings keys above in `--settings`. Rank tool sizes. Verify whether `--strict-mcp-config`
   blocks claude.ai connectors and whether `disableClaudeAiConnectors` is needed. Check each
   setting key exists in this version; cite findings, don't assume. Output
   `docs/spikes/NN-lean-context-findings.md` with a per-role recommended tool list.
2. **Background agents (build).** Per role, pass the minimal toolset so definitions are removed,
   not just denied: worker Bash, Read, Edit, Write, Glob, Grep (+ Agent if the spike shows the
   workers use it); manager and product-manager Bash, Read, Glob, Grep. Add the settings keys
   (no bundled skills, workflows, artifacts, connectors; keep the project's bridle skills). A
   per-role `[roles.*] tools` key in config.toml overrides. Report each role's starting context
   in the spike's table before and after.
3. **The appended prompt.** Check the preamble, role file and CLAUDE.md for repetition; keep
   what an agent needs on turn one, point to the rest.
4. **Orchestrator and advisor (interactive).** `scripts/claude-orchestrator` and
   `scripts/claude-advisor` pass `--settings` with the same disables **except Remote Control**
   (the human reaches them through it), `--strict-mcp-config`, and bare-name denies for tools
   they don't use (AskUserQuestion, plan mode, DesignSync, NotebookEdit, PushNotification,
   ReportFindings, Artifact...). Keep Agent, ToolSearch, web search/fetch, Cron (the heartbeat,
   until fx7x replaces it) and whatever fx7x's wait command needs.
5. **The orchestrator's state file** shrinks to current state; history lives in git and
   `docs/context/role-notes.md`. (The orchestrator does this at its next handover.)
6. **Track it.** Record each session's starting context and growth (fx7x's context slice logs
   it), so "how long can the orchestrator run" has a number.

Workers must never touch the live orchestrator session or the human's `~/.claude/settings.json`.
