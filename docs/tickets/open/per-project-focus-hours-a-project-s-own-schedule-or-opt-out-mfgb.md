---
id: mfgb
title: "Per-project focus hours: a project's own schedule or opt-out"
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq, phyy]
---

## The ask


The human, 2026-10-01 (via the NUC's orchestrator, m-3169), verbatim: "I have implemented global
focus time in bridle, but that needs to allow for project-specific config. I don't want to be
locked out of my notes concierge during the day. I want a different schedule for that project."

## Today

Focus hours (cvaq) are machine-wide: the `[[focus]]` entries in `~/.bridle/config.toml`.
`focus::refuse_advisor_if_locked` and the `bridle focus gate` hook apply to every project's
sessions.

## Wanted

A project can set its own focus schedule, or opt out, in its project config. The machine-wide
schedule stays the default. First user: the `notes` concierge (phyy).

Constraint: only the human sets focus hours (the orchestrator role never edits
`[[focus]]` or `focus-override.toml`). A project-level setting must keep that true, so it can't
live anywhere an agent may edit unreviewed (e.g. a project's `.bridle/config.toml` that agents
commit to). Where it lives is the main design question.
