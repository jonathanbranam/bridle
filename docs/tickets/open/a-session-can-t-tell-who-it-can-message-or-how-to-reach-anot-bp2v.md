---
id: bp2v
title: A session can't tell who it can message, or how to reach another machine
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [cy2v, r8kv, sk7p]
tasks: []
---

## The ask


The human, verbatim (2026-10-03, via the advisor):

> Also - related the advisor there had no idea how to send messages to you or who the valid
> recipients were. That needs to be improved in their prompt

(The advisor on the NUC's notes project managed to send a test to dalek's advisor on the notes
daemon, but without knowing the addresses.)

## Today (advisor, checked 2026-10-03)

- `bridle send --help` names only "an agent id/name, `human`, or `role:<name>`". It doesn't
  mention external principals (`external:orchestrator`, `external:advisor`,
  `external:advisor/<name>`) or visitors from another machine (`<name>@<machine>`), nor that a
  message lands only on the daemon you send it to (`--project`).
- `bridle agents` lists agents only; `bridle token list` (which knows the external principals and
  visitors) is human only. Nothing lists who a session can message.
- The advisor role (`workflow/base/roles/advisor.md`) names only `external:orchestrator`,
  `external:mail` and "the agent that asked"; nothing about other machines.

## Wanted

- **Prompt (quick):** the advisor and orchestrator roles say who they can message and how: the
  principal forms above; for another machine, `bridle --project <p> send <principal>` to that
  project's daemon, using the visitor token in `credentials.toml`; and that the recipient sees
  it only if it watches that daemon (until cy2v).
- **Tool:** a way to list valid recipients for a project (agents, external principals, visitors),
  readable by any principal, e.g. `bridle send --list` or `bridle principals`.
