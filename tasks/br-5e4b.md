+++
id = "br-5e4b"
title = "Trim agent context growth: read by range, cap check/git output (ct8m step 3 build)"
kind = "chore"
state = "planned"
created_at = "2026-09-29T18:20:36.018Z"
updated_at = "2026-09-29T18:33:32.011999Z"
size = "S"
summary = "Trimmed agent context by adding reading and output guidelines to role prompts and docs index. Removed duplicated 'no memory' language from CLAUDE.md (now source of truth is the system prompt). Clarified commit expectations in worker role. All checks passed."
+++

Source: ranked recommendations 3, 4, 5 and 7 in docs/spikes/08-lean-context-findings.md (section 'Ranked recommendations', landed 344eb74; read that section and the file-cost table before you start). Goal: role prompts and rules tell agents to read by range and cap chatty output, with no behaviour change. Do: 1) workflow/base/roles/worker.md (and manager.md/product-manager.md where the same reads apply): add a short 'Reading and output' rule set: read CHANGELOG.md with head -30 (entries go on top), read cli.rs/commands.rs/other big files by sed -n or Read with offset and limit, read one design doc not the folder (cat docs/design/*), cap check output with | tail -n 30, git output with -n / --stat; keep it to a few lines, in the style of the surrounding text. 2) A pointer index so agents can pick the right doc without shotgun reads: a compact 'which doc covers what' table in docs/README.md if one is missing (check first) and reference it from the worker role, no duplication. 3) The worker check instruction (workflow/base/skills/worker/SKILL.md handoff): show the tail of check output. 4) Repetition check of the appended prompt: crates/bridle-daemon system-prompt template (system-prompt.md, find it) vs CLAUDE.md: remove the duplicated just check / just check-affected and no-memory lines from whichever side is not the source of truth, and resolve the 'Don't commit unless asked' contradiction for agents (CLAUDE.md is the human's file for their own sessions: prefer adding to the worker prompt 'commit on your branch as your task says; this overrides CLAUDE.md's commit line' over editing CLAUDE.md). Update any prompt snapshot tests. Docs and prompt text only, no Rust logic change beyond snapshot/tests. Acceptance: just check passes. Model: Haiku. Out of scope: --tools whitelist (br-9fca), long-lived role compaction (item 6), orchestrator/advisor scripts (step 4, held).

## Thread

### note · agent:lean-prompts · 2026-09-29T18:33:32.011Z
done: added reading/output rules to roles and docs index, removed CLAUDE.md duplicates, clarified commit expectations; cdc2969
