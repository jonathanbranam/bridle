---
id: kuvh
title: "Remove 'bridle agent wake --all-projects' once 3haz lands and rolls out: warn first, then delete"
kind: chore
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: [3haz]
see: [cy2v]
tasks: [br-kuvh]
---

## The ask

The human, verbatim (2026-10-04, via the advisor), on
[[one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v|cy2v]]'s
`bridle agent wake --all-projects` (task br-1ddd, planned) once
[[daemons-deliver-mail-to-each-other-across-machines-store-and-3haz|3haz]] lands:

> Leave all-projects for now. It'll be a NOOP after this lands. We can warn and remove later. Add
> a follow up ticket to remove it after this lands and rolls out so we don't forget

Why it becomes a no-op: under 3haz every daemon forwards a principal's messages to its home
daemon, and every orchestrator wake is a message, so one waiter on the home daemon sees
everything and watching every daemon at once adds nothing.

**Not before** 3haz has landed **and rolled out** (every machine's daemons upgraded, the NUC
included), so no daemon still needs the fan-out.

## Steps

1. **Warn:** `--all-projects` still works but prints a deprecation note to stderr ("not needed
   since 3haz: one waiter on your home daemon sees every project"); the role prompts
   (`workflow/base/roles/`) stop using it.
2. **Remove**, a release or so later: the flag, its code and tests; `docs/design/cli.md`;
   any role text still naming it. Resolve cy2v.
