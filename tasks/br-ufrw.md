+++
id = "br-ufrw"
title = "Auto mode's classifier context per machine for bridle projects: who writes autoMode.environment, and how it stays current"
kind = "question"
state = "integrated"
created_at = "2026-10-04T12:23:53.432Z"
updated_at = "2026-10-04T12:32:51.922706Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/automode-study"
commit = "fcb52264d78d84bfb2396417286f7b5735a152af"
summary = "Added a Recommendation to ticket ufrw: launch `bridle session` with a generated `--settings` autoMode.environment (derived workspace paths + machine `~/.bridle/config.toml` + project `[auto_mode]`), don't write ~/.claude/settings.json (optional print-only command for plain claude). Finding: spawned agents run dontAsk/acceptEdits with --setting-sources project, so only the human's interactive sessions are affected. Smallest slice, files, rejected alternatives, migration, and unverified points (no live run) are in the ticket. Recommend conservative variant: project config may only tighten, trust lines from machine file."
+++

original id: ufrw
Investigation, no code. Ticket: docs/tickets/open/auto-mode-s-classifier-context-per-machine-for-bridle-projec-ufrw.md (read it: background and questions 1-4). The human 2026-10-04, relayed by track-web-f5: 'investigate how to configure Claude Code auto-mode settings for bridle projects on a per-machine basis.'
Deliverable: a recommendation written into the ticket (a new 'Recommendation' section; docs, not code), answering Q1-Q4: should bridle generate the machine-level autoMode.environment block for the human to review, or launch sessions and agents with a generated --settings file per project (bridle already builds --settings for spawn and 'bridle session', see crates/bridle-daemon spawn and crates/bridle/src/session.rs, and docs/design/workflow-layers.md 'Layer hooks'); how project facts stay apart from machine facts; how it stays current when workspaces move. Verify against the Claude Code docs (https://code.claude.com/docs/en/auto-mode-config.md) and docs/spikes/01-stream-json-findings.md, citing sources; say what you could not verify rather than assume, and do no live run. Pick one recommended design with its smallest first slice, name the files it would touch, the rejected alternatives and why, and a migration plan for existing projects (rule: changes to projects' files need one; never change an existing project's files without the human's review, see rule existing-projects). Do not edit ~/.claude/settings.json.
Acceptance: the ticket has the section; bridle ticket check passes. Model: Sonnet. Out of scope: building it; the human reviews the recommendation first.

## Thread

### note · external:orchestrator · 2026-10-04T12:23:59.255Z
The human 2026-10-04, relayed by track-web-f5: 'investigate how to configure Claude Code auto-mode settings for bridle projects on a per-machine basis.'

### note · agent:automode-study · 2026-10-04T12:32:46.254Z
done: Recommendation added to ufrw (per-launch --settings in bridle session, no global-file writes; spawned agents don't use auto mode as built); docs only; d2d5c27f

### note · agent:manager-2 · 2026-10-04T12:32:51.922Z
integrated: fcb52264d78d84bfb2396417286f7b5735a152af (branch bridle/automode-study)
