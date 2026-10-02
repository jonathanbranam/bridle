---
id: sfb3
title: Orchestrator and advisor session names say which machine they're on
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [where-the-single-orchestrator-lives-hj4g]
closed: 2026-10-02T00:43:36.513986Z
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> small issue - when I run orchestrator on the NUC it should have a unique name so I can
> differentiate it in Claude mobile. (Also for advisor)

## Today (at 0a5c4d6)

`scripts/claude-orchestrator` passes `--name bridle-orch --remote-control bridle-orch`;
`scripts/claude-advisor` passes `--name bridle-advisor --remote-control bridle-advisor`. The same
names on every machine, so the laptop's and the NUC's sessions look identical in Claude mobile.

## Shape

Add the short host name to both names, e.g. `bridle-orch-nuc`, `bridle-advisor-<laptop>`
(`hostname -s`, lowercased). Optionally a `BRIDLE_SESSION_SUFFIX` override for a friendlier name.

## Several advisors at once (2026-09-29)

The human, verbatim:

> I do want multiple advisors yes. I can name them when I start them. Many would be short lived.
>
> For now KISS one inbox shared among advisors but my concern is primarily that the advisor
> should be aware that they are not the only one. They should probably include their name in any
> message.
>
> But an advisor will be ephemeral, except for the main one, so I think from a credential aspect
> they all serve the same role.

So:

- `scripts/claude-advisor [name]`: an optional name, added to the session name
  (`advisor-<name>`, or `advisor` with no name; suffix with `BRIDLE_SESSION_SUFFIX` for machine naming).
- One identity, one token, one shared inbox: all advisors are `external:advisor`. No per-advisor
  principals.
- `workflow/base/roles/advisor.md`: other advisors may be running at the same time. Each signs
  its messages with its name (e.g. "From advisor (research): ..."); the name comes from the
  script (e.g. `BRIDLE_ADVISOR_NAME`). The shared inbox and working copy are the known costs: an
  advisor doesn't assume a message it didn't send was its own, and commits only its own ticket
  files (`git add <file>`, never `-A`).

## Session name design (2026-09-29, applied)

Shorter names without 'bridle-' prefix:

- Orchestrator: 'orch' by default; 'orch-<suffix>' when `BRIDLE_SESSION_SUFFIX` is set (replacing the host-based default from br-47ba).
- Advisor: 'advisor' by default; 'advisor-<name>' when a name is given; can be suffixed with `BRIDLE_SESSION_SUFFIX` for machine naming.

## Resolution

Resolved by: br-47ba (8a1c3d4), br-df68 (a443618)
