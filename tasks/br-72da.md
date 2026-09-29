+++
id = "br-72da"
title = "Lean launch for the orchestrator and advisor scripts (ct8m step 4)"
kind = "chore"
state = "planned"
created_at = "2026-09-29T20:03:41.549Z"
updated_at = "2026-09-29T20:05:31.394554Z"
size = "S"
+++

Ticket: docs/questions/open/*ct8m.md, plan step 4; docs/spikes/08-lean-context-findings.md (which settings keys exist in the installed Claude Code and what each saves; its bare-name deny list and the per-role table). The human decided (2026-09-29): KEEP AskUserQuestion in both. Files: scripts/claude-orchestrator (already changed by fx7x slice 1a: pid file and SessionStart hook; read the current version and keep those parts intact) and scripts/claude-advisor. Goal: both interactive sessions start leaner. Add --settings keys the spike confirms exist (disableBundledSkills, disableWorkflows, disableClaudeAiConnectors, disableArtifact; NOT disableRemoteControl: the human reaches both through Remote Control), --strict-mcp-config, and bare-name denies (permissions.deny) for tools these roles never use per the spike's usage counts: EnterPlanMode, ExitPlanMode, DesignSync, NotebookEdit, PushNotification, ReportFindings, RemoteTrigger, Artifact, and any others the spike shows as unused. Keep: AskUserQuestion, Agent (the watcher can be a subagent), ToolSearch, WebSearch/WebFetch, Bash background execution and Monitor (bridle wait-for-wake runs as a background command), Cron tools (heartbeat until wait-for-wake is in use; read workflow/base/roles/orchestrator.md), and the project's bridle skills. Merge with, don't clobber, the SessionStart hook --settings the orchestrator launcher already passes (one JSON object, or the mechanism the spike/07 findings show works for combining). Verify by a cheap scratch check that does NOT touch the live sessions: run the same flag set on Haiku in a scratch dir and record the starting context before and after, in a short section appended to docs/spikes/08-lean-context-findings.md (under 0.5 USD). Docs: the scripts' header comments, docs/context notes if they describe the launch flags. Acceptance: just check passes (shellcheck-style sanity: bash -n on both scripts); numbers recorded. Model: Sonnet. Out of scope: state-file shrink (step 5), tracking (step 6), never launch or restart the real orchestrator or advisor; the human restarts them to pick this up.

## Thread

### note · agent:pm-1 · 2026-09-29T20:05:31.394Z
HELD: the human has NOT decided whether AskUserQuestion stays; the brief's 'keep AskUserQuestion' was wrongly recorded (m-1906). Do not start until the orchestrator releases it with the human's answer.
