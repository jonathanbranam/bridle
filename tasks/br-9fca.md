+++
id = "br-9fca"
title = "Background agents get a minimal toolset and lean settings, per-role overridable (ct8m step 2)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T17:41:26.870Z"
updated_at = "2026-09-29T17:41:30.515586Z"
+++

Ticket: docs/questions/open/*ct8m.md, Plan step 2, and the findings of the spike task blocking this one (docs/spikes/NN-lean-context-findings.md, read it first: its per-role tool list and the setting keys that exist in the installed Claude Code are the spec, not the ticket's guesses). Goal: spawned agents carry only the tool definitions their role needs, removed not just denied. Change crates/bridle-claude/src/command.rs and crates/bridle-daemon/src/config.rs (role defaults; the role permission code near lines 275-415) to pass the spike's per-role tool set (--tools or bare-name denies, whichever the spike shows works) and to add the recommended settings keys via --settings, keeping the project's bridle skills. A per-role [roles.*] tools key in .bridle/config.toml overrides the default. Document in docs/design/agent-host/ (roles-and-config.md, and command construction where it is described) and CHANGELOG. Report each role's starting context before and after in the task summary. Acceptance: just check passes; unit tests for the generated command line per role and for the override; one cheap live check (Haiku, scratch, under 0.5 USD) that a spawned worker still runs bridle and edits files. Model: Sonnet. Out of scope: the appended prompt (step 3), orchestrator/advisor scripts (step 4), state file (step 5).
