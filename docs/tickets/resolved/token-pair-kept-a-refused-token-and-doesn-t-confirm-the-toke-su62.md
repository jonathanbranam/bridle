---
id: br-su62
title: token pair kept a refused token, and doesn't confirm the tokens work afterwards
kind: bug
created: 2026-10-14
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: [br-sh8y]
related: [br-nbgp, br-378y]
resolved: 2026-10-15T20:52:32Z
---

The human, 2026-10-14 ~7:45 AM ET (to aide), verbatim: "Can we improve token pair to actually check and see if the tokens are accepted?"

## What happened (2026-10-14)

- ~6:49 AM ET vesta's meta-notes daemon upgraded to v0.7.2 and began refusing the orchestrator's
  token ("unauthorized: invalid token"; cause br-nbgp).
- ~7:30 AM ET the human ran `bridle token pair --projects meta-notes` on dalek. It printed
  "machine dalek (role tokens): minted 0, kept 5 / remus: minted 0, kept 5 / vesta: minted 0, kept 4",
  so it judged every entry working, though the orchestrator's was refused.
- `--rotate` (minted 5/5/4) fixed it: the orchestrator confirmed its `[orchestrator.vesta]`
  meta-notes token is accepted afterwards (m-12615).

docs/design/cli.md says an entry is kept only when it works (`GET /v1/status`), so either that
check passed for a token vesta's daemon refuses, or it checks something other than the
holder's entry against the daemon the holder talks to. Find which.

## The ask

1. The "kept" check must reject a token the target daemon refuses, so a plain run re-mints it
   (no `--rotate` needed for this case).
2. After minting and storing, token pair checks every entry it wrote or kept, from the machine
   that holds it, against the daemon it's for, and prints the result per role and project
   (accepted / refused, with the reason). Exit non-zero if any is refused.

## Design: a direct check command (the human's go)

The human, 2026-10-14 ~8:00 AM ET (to aide), verbatim: "What test request is used? Should we
just have a command to directly check whether a token is valid?" After aide proposed the
command below: "Yes please".

How the check works today: token pair runs the hidden `bridle token pair-check --role R
[--machine M]` over ssh on the holder's machine. It reads the holder's credentials entry,
resolves the project's daemon URL from the holder's own registry, and calls `GET /v1/status`
(a visitor token must also report a `visitor_home`). Lead to check: dalek's registry still says
`meta-notes = { machine = "nuc", ... }` (rename leftover, br-r2nr), so the URL it resolves for
a vesta project may not be the daemon that refuses the token.

- **`bridle token check [--machines m,..] [--projects p,..] [--roles r,..]`**: read-only (mints
  and writes nothing). For every credentials entry selected (default all), it checks the entry
  from the machine that holds it, against the daemon it's for, and prints one line per machine,
  role and project: accepted, or refused with the reason (invalid token, no home, unreachable,
  no entry). Exit non-zero if any is refused. `--json` for agents.
- **token pair uses the same code**: for its keep-or-mint decision, and as its final step (check
  everything it wrote or kept, print the same lines, exit non-zero on any refusal). One check,
  so the two can't disagree.
