---
id: k3wp
title: Bridle removes an agent and its worktree when its work lands, not the manager by hand
opened: 2026-09-29
resolved: 2026-09-29
repos: [bridle]
changes: [b0b9ab6, 61c52ff, ab158f6, e26c54d]
specs: []
needs: []
see: [tr7k, sq4m]
closed: 2026-09-30T05:12:44Z
---

## What happened

`python-pack-2` sat stopped for hours after its work was merged (761fcd5). It had been spawned
to carry on another agent's branch and worktree (`bridle/python-pack`, `wt/python-pack`), with
an unused branch and worktree of its own (`bridle/python-pack-2`). The manager's merge step,
`bridle rm <name> --delete-branch` for the branch it merged, removed the agent named after that
branch and missed this one. A daemon restart had also stopped it mid-check, and nobody resumed
it. `bridle agents` hides stopped agents, so only `bridle tui` showed it.

The human, verbatim (2026-09-29): "This needs to be done consistently; does the manager make
these decisions for every merge?"

Today it is the manager's judgement at each merge, from one line of its role prompt. Nothing in
bridle checks it happened.

## Proposal

- **Make landing mechanical.** `bridle task done <id> --commit <sha> --branch <branch>` (tr7k)
  already names the branch. Have it (in the daemon) remove every agent whose worktree is on that
  branch, delete the worktree and the branch (it's merged), and record what it removed on the
  task. The manager then runs one command, not a sequence it can get partly wrong.
- **One agent per branch.** Continuing work on an existing branch should resume or renew that
  branch's agent, not spawn a second one onto another agent's worktree; if a new agent is
  needed, it takes over the branch and the old agent is removed then. So "the agent for this
  branch" is always one agent.
- **A safety net.** `bridle status` (or the daemon on startup) lists stopped agents whose
  branch is merged, so leftovers show up instead of hiding.

## Resolution

Resolved by 61c52ff (br-7d81, b0b9ab6): `bridle task done --branch` removes the landed branch's agents, worktree and branch, and `bridle status` lists stopped agents whose branch has merged; and e26c54d (br-0589, ab158f6): spawn refuses a branch another agent holds, so a branch has one agent. The answer lives in docs/design/cli.md.
