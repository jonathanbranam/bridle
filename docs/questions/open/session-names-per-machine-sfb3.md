---
id: sfb3
title: Orchestrator and advisor session names say which machine they're on
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [where-the-single-orchestrator-lives-hj4g]
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
