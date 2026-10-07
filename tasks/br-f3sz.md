+++
id = "br-f3sz"
title = "Retire scripts/claude-orchestrator and claude-advisor, step 2: delete the scripts and their mentions (w3pk)"
kind = "chore"
state = "integrated"
created_at = "2026-10-06T23:09:42.775Z"
updated_at = "2026-10-07T05:53:20.923216Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/scripts-f3sz"
commit = "0d6f16defe06ac70192d0bfe97af906875954570"
summary = "Deleted scripts/claude-orchestrator and scripts/claude-advisor. Updated code comments and docs (cli.md, orchestrator-supervision.md, config.rs, session.rs, commands/orchestrator.rs, mail/local.rs) to reflect that the functionality is now in bridle session commands and the daemon supervisor. Left backward-compatibility code in config parsing and tests. Added CHANGELOG entry."
ticket = "w3pk"
+++

Step two of ticket w3pk (docs/tickets/open/retire-scripts-claude-orchestrator-and-claude-advisor-the-or-w3pk.md); step one is br-w3pk (the supervisor relaunches with `bridle session orchestrator --project <project>`).
PRECONDITION, check before you start: br-w3pk is integrated AND the running daemon is a build that has it (`bridle status` shows the daemon's version/build, compare with the merge commit of br-w3pk; or check the daemon was restarted/self-upgraded after it). If not, stop and say so on this task's thread: do not delete anything, because the running daemon would still type the script's path.
Do: delete scripts/claude-orchestrator and scripts/claude-advisor; update the remaining mentions: docs/design/cli.md (`orchestrator note-session`), docs/design/agent-host/orchestrator-supervision.md, code comments in crates/bridle/src/session.rs, crates/bridle/src/commands/orchestrator.rs, crates/bridle/src/cli.rs and crates/bridle-mail/src/local.rs, plus crates/bridle-daemon/src/config.rs if it still names the script. `grep -rn 'claude-orchestrator\|claude-advisor'` outside CHANGELOG, resolved tickets, spikes and docs/context must find nothing. Leave CHANGELOG history, resolved tickets, spikes and docs/context alone. Add a CHANGELOG entry (read it with a limit; entries go on top).
Acceptance: just check passes; the grep above is clean.
Model: Haiku (mechanical). Out of scope: any behaviour change.

## Thread

### note · agent:scripts-f3sz · 2026-10-07T05:52:28.119Z
done: scripts deleted, mentions updated in code and docs, 1291 tests passing; 59e33aac

### note · agent:scripts-f3sz · 2026-10-07T05:52:32.571Z
done: scripts deleted and mentions updated; 1291 tests pass; 59e33aac

### note · agent:manager-2 · 2026-10-07T05:52:38.577Z
integrated: 0d6f16defe06ac70192d0bfe97af906875954570 (branch bridle/scripts-f3sz)

### note · agent:manager-2 · 2026-10-07T05:53:20.923Z
cleanup: removed agent scripts-f3sz, branch bridle/scripts-f3sz
