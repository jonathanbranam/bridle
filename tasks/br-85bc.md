+++
id = "br-85bc"
title = "mrhe 1: 'bridle session orchestrator|advisor' replaces the launch scripts, runnable from any directory"
kind = "feature"
state = "planned"
created_at = "2026-09-30T05:20:27.654Z"
updated_at = "2026-09-30T05:20:30.827452Z"
+++

Ticket: docs/tickets/open/bridle-without-a-clone-of-its-repo-mrhe.md (Shape, third bullet; read it). The launch scripts scripts/claude-orchestrator and scripts/claude-advisor become 'bridle session orchestrator|advisor [name]' (crates/bridle), runnable from any directory, so a project needs no bridle clone for them. Keep every behavior the scripts have now: project-aware session names and remote-control names with the host suffix (BRIDLE_PROJECT, BRIDLE_SESSION_SUFFIX, advisor [name]), pane tag via 'bridle pane tag', refusing under a bridle agent unless the test flag is set, the tools-only refusal (hw6c 2), the advisor pid file and BRIDLE_ADVISOR_NAME, bash-free (works on Linux and macOS). The role prompt comes from 'bridle prime' (which already reads the resolved workflow), not a path relative to a clone. Keep the scripts as thin wrappers that exec the new command (compat, remove later). Read the scripts first and port their tests (stub claude/tmux on PATH). Docs (cli.md, docs/context/nuc-host.md, role files that cite the scripts), CHANGELOG. Acceptance: just check passes; same command lines as the scripts produced for orchestrator, advisor and named advisor, shown in the summary. Model: Sonnet. Out of scope: workflow vendoring (mrhe 2), installing without cargo, the tmux-pane advisor start (ervd 2).
