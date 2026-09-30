---
id: a67t
title: Group bridle's 54 top-level commands into subcommands; split the CLI's big modules
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [shell-completions-for-bridle-6rh7]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> Also - bridle has WAAAY too many top-level commands; we need to analyze this and group into
> commands and subcommands it's getting way out of hand. Also, for braeaking up the large module,
> can we ship multiple binaries that are called separate, I think git does this

## Today (at ac29ab1)

54 top-level commands (`bridle --help`):

```
serve stop-daemon doctor init launchd rebuild daemons status spawn agents show send inbox
interrupt stop resume renew rm logs events wait usage cost tui budget token task impact probe
land conflict port dep ask answer claim release ready queue statusline stop-check arch-guard
orchestrator wait-for-wake handover prime rules sync spec goals arch explore trace help
```

`crates/bridle/src`: `commands.rs` 3,578 lines, `cli.rs` 2,248; the crate is about 10k lines.
Some commands are called by Claude Code hooks and scripts, not people: `statusline`,
`stop-check`, `arch-guard`, `wait-for-wake`, `orchestrator note-session`.

## A first grouping (the advisor's draft, for the analysis to test)

| Group | Today's commands |
|---|---|
| `daemon` | serve, stop-daemon, doctor, init, launchd, rebuild, daemons |
| `agent` | spawn, agents (list), show, interrupt, stop, resume, renew, rm, logs |
| `task` | task, claim, release, ready, queue, dep, land, conflict, impact, ask, answer |
| `usage` | usage, cost, budget |
| `orchestrator` | orchestrator, handover, prime, wait-for-wake |
| `workflow` | rules, sync, spec, goals, arch, explore, trace |
| `hook` (hidden) | statusline, stop-check, arch-guard |
| top level | status, send, inbox, events, wait, tui, token, port, probe |

The daily ones (`status`, `send`, `inbox`, maybe `agents`, `queue`) could keep short top-level
aliases.

## Multiple binaries, git-style

Git dispatches `git foo` to a `git-foo` program on the PATH. For bridle, the advisor's read:
it doesn't fix what the human is after, and adds cost.

- The size problem is two files an agent has to read in slices. Splitting `commands.rs` and
  `cli.rs` into one module per group (`commands/task.rs`, `commands/agent.rs`, ...) fixes that
  inside one binary, and follows the grouping above.
- Build time: Rust compiles one crate as a unit, so separate binaries only help if each is its own
  crate, and most of the build is `bridle-daemon` and dependencies either way.
- Several binaries mean several things to install and keep at the same version on every machine
  (laptop, NUC), and a version skew between them is a new class of bug.

So: one binary, modules per group. Revisit separate binaries only for a real need (e.g.
plugins from other repos).

## Churn

Role prompts, skills, `scripts/`, hook settings, `docs/design/cli.md` and the tests all name
commands. Keep the old names as hidden aliases for a release, update every reference in the same
change, then drop the aliases. Do this before 6rh7 (completions), or regenerate after.
