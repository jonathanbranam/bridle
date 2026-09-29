+++
id = "br-60ec"
title = "Spike: measure each role's starting context; lean toolsets (ct8m step 1)"
kind = "research"
state = "integrated"
created_at = "2026-09-29T17:41:26.855Z"
updated_at = "2026-09-29T17:59:00.863919Z"
branch = "bridle/lean-spike"
commit = "dfd3fa8"
summary = "Spike 08 (docs/spikes/08-lean-context-findings.md), Claude Code 2.1.284, 85 Haiku runs in a scratch dir, $0.88. A spawned agent's first turn is ~19-20K, not 48-60K, and most built-in tool definitions are deferred (11K, not in the total), so ct8m's premise is mostly wrong. `--tools <whitelist>` is the lever: worker `Bash,Read,Edit,Write,Glob,Grep` 19.1K to 12.8K; manager and product-manager `Bash,Read,Glob,Grep` +`Agent` 19.7K/19.4K to 15.4K/15.1K (12.6K/12.3K without Agent). Bare-name denies and settings keys save 1-2K and add nothing once --tools is passed (it drops Skill, so no skill listing). --strict-mcp-config already blocks the claude.ai connectors; disableClaudeAiConnectors is not needed for spawned agents. All settings keys exist; disableArtifact/disableRemoteControl do nothing in -p. Workers' Monitor use is replaceable by Bash run_in_background (tested). Unmeasured: interactive orchestrator/advisor shape (would load the human's settings), the gap to 48-60K seen live."
+++

Ticket: docs/questions/open/*ct8m.md (read all; the Plan step 1 is this task). Spike style: docs/spikes/01-stream-json-findings.md, docs/spikes/05-stop-hook-findings.md, get_context_usage in the accurate-context work (kc4v; grep the crates for it). Live runs are authorised: Haiku only, in a scratch dir outside the repo, a few dollars at most in total (stop and report if it goes past ~4 USD); never touch the live orchestrator or advisor sessions or the human's ~/.claude/settings.json. Measure the starting context (first turn) of a worker and a manager spawn built exactly as bridle builds them (crates/bridle-claude/src/command.rs, crates/bridle-daemon/src/config.rs) under: today's flags; plus --tools with a whitelist; plus bare-name --disallowedTools / permissions.deny; plus each settings key named in the ticket (disableBundledSkills, disableWorkflows, disableClaudeAiConnectors, disableArtifact, skillOverrides; disableRemoteControl only for reference) passed via --settings. Rank tool definition sizes. Verify whether --strict-mcp-config blocks the claude.ai connectors (Gmail, Drive, Calendar, Claude Docs) and whether disableClaudeAiConnectors is needed. Check each settings key exists in the installed version (claude --version) and cite what you observed. Also record starting context for the interactive orchestrator/advisor launch shape only if it can be done without touching the live session (a scratch interactive-equivalent run); otherwise note it as unmeasured. Deliverable: docs/spikes/NN-lean-context-findings.md with a before/after table per role and a recommended tool list per role (worker, manager, product-manager, plus which of Agent, Monitor and ToolSearch each role really uses: check the roles and a sample of recent transcripts under ~/.claude/projects for tool usage counts). Docs only. Acceptance: just check passes. Model: Sonnet. Out of scope: any code change, steps 3-6 of the ticket, the orchestrator and advisor scripts.

## Thread

### note · agent:lean-spike · 2026-09-29T17:58:36.006Z
done: spike 08 — first turn is ~19K not 48-60K; most tool defs are deferred; --tools whitelist is the lever (worker 19.1K→12.8K, manager/PM →15.4K/15.1K with Agent); strict-mcp already blocks claude.ai connectors; Haiku spend $0.88; 9e2cc0f; docs: docs/spikes/08-lean-context-findings.md, ct8m ticket pointer

### note · agent:manager-2 · 2026-09-29T17:58:42.299Z
integrated: dfd3fa8 (branch bridle/lean-spike)

### note · agent:manager-2 · 2026-09-29T17:59:00.863Z
cleanup: removed agent lean-spike, branch bridle/lean-spike
