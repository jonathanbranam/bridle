---
id: memory.none
severity: must
roles: [human, orchestrator, manager, worker, reviewer, designer]
locked: true
---
Never use assistant memory. Claude Code's auto memory is off (settings
`autoMemoryEnabled: false` and `autoDreamEnabled: false`), and no agent keeps
notes anywhere outside the repository.

Anything worth keeping goes into the repository, where the human and every
other agent can read, review and correct it:

- a decision or design fact: the design docs, or a question ticket's
  resolution;
- something not yet settled: a question or spike ticket;
- how agents should work: a rule in the workflow layers (this file's kind);
- something for the person who gave you the task: a message.

Why: the human's words, 2026-09-27: "don't save memories! That is an
ultimatum for this entire project and every bridle-based project." Memory is
invisible to the human and to other agents, can't be reviewed, and goes
stale. Git is the source of truth (decision 2).

Enforced by: `--settings` on every agent bridle spawns
(`crates/bridle-claude/src/command.rs`, in the `bridle` repo),
`.claude/settings.json` in each project, and the bridle preamble.
