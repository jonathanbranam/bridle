---
id: 2bzw
title: How strong should agent provenance be?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [xqvg]
---

## The question

From `docs/agent-host.md` §5.3:

> On one machine as one user, a token file is readable by any process of that
> user, so provenance is **attribution that honest agents can't get wrong by
> accident, not a security boundary**.

And §4.9, added in the v1 build:

> **An agent's token is also kept in `.bridle/agents/<id>/token` (0600)** so
> `resume` can re-inject the same identity (the store keeps only hashes).

## Why it matters

The human asked that human actions be distinguishable from agent actions
(the v1 request, item 7). Today any agent can read `.bridle/tokens/human`, or
another agent's token file, and act under that identity. Honest agents won't
do this, and the preamble tells them not to. But once a worker's prompt comes
from untrusted content (issues, web pages), attribution could be forged.

## Notes

Options, roughly in order of cost:

- Keep it as is. It's documented as attribution.
- Keep agent tokens only in daemon memory and re-mint on resume. This drops
  the token files, but a token still sits in each agent's environment.
- A PreToolUse hook or `--disallowedTools` pattern that blocks reads under
  `.bridle/`.
- Run agents as a separate Unix user or in a sandbox. That's the only real
  boundary, and it's also the remote-host story
  ([[finding-remote-daemons-from-the-laptop-xqvg|finding remote daemons]]).
