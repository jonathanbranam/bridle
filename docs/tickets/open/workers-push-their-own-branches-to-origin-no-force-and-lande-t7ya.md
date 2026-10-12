---
id: br-t7ya
title: Workers push their own branches to origin (no force), and landed branches are cleaned up safely
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-ehg7, br-kt25]
---

## The ask

The human, 2026-10-12 ~10:36 AM ET (to advisor product-manager, dictated, in the grilling session on kx5u/kt25), verbatim:

> Not part of project moves. I don't know which epic or theme this fits in, but I do want to do the one thing that's from V2, which is that workers should push their branches. Frankly, branches are cheap. They're cheap on local, and they're cheap on remote. The most expensive thing about a branch for us is the build artifacts, but those are not pushed.
>
> I don't see any reason an agent cannot push its own branch. A worker agent, it's safe. It can't hurt anything else. They shouldn't be force-pushing, I don't think, ever. They need to work with the manager if they need some sort of thing like that. I think we already have that workflow, okay, but there's no significant reason why a worker (and I understand it) can't push its own branch.
>
> That would help me out for several reasons: worker branches are visible. If I'm on GitHub, I can go see what's going on. Also, if they're asking me to review code, I don't think we need a PR process with GitHub for our internal work. I don't really see benefit to it yet, but if it's truly something that a human needs to review, then a PR would be the right approach. That comes later.
>
> I do want to put in the reliability hardening thing that workers should start pushing their branches. The consequence of that is we're going to have to clean up those branches on the remote, and we want that to be very, very safe and very limited. That would maybe be the only argument I could see for using an official PR: we could turn on the "delete branch after merge" option, and that would automatically clean things up without us having to build our own machinery.

## The ask (PdM reading; the human reviews before design or build)

1. **A worker pushes its own branch** (`bridle/<name>`) to origin as it works: never a force push,
   never any other branch, never the integration branch. Anything needing a force push or a
   rewrite goes through the manager.
2. **Remote branches are cleaned up safely**, in a very limited way: only a worker's own
   `bridle/*` branch, only after its ticket has landed or been dropped.
3. **Why:** worker branches are visible on GitHub to the human; work survives a lost machine
   (br-ehg7) and is the first step of V2 / roadmap phase 3 (workers anywhere); it costs little
   (build artifacts are not pushed).

## Today (facts, 2026-10-12)

- Workers can't push at all: `Bash(git push *)` is in their built-in `disallowed_tools`
  (docs/design/agent-host/operating-model.md; crates/bridle-daemon/src/config.rs). Only the human,
  the orchestrator and managers run `bridle push`, for the integration branch, from the owner's
  clone (pre-push hook, rule `one-pusher-for-the-integration-branch`).
- The manager removes the local worker branch at cleanup after landing (`cleanup: removed agent
  ..., branch bridle/...` on ticket threads); a pushed branch would need the same on origin.

## For the design

- Cleanup: bridle deletes the remote branch at the same cleanup step, or GitHub's "automatically
  delete head branches". Fact to weigh: GitHub's option fires only when a PR is merged on GitHub.
  Bridle lands by a local squash and pushes the integration branch, so GitHub would not see a
  merge and would not delete the branch unless landing went through the PR. PRs are "later" (the
  human), only for code a human must review.
- The pre-push hook today guards the integration branch only; the worker's deny rule would narrow
  from all pushes to "own branch, no force".
- Not part of project moves (kx5u): moves in the MVP and V1 still land all work first.
