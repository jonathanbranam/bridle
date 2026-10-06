---
id: 95mu
title: "A change spec (proposal and design) reviewed for risk and impact before any worker builds: a real gate, not a convention"
kind: arch-revision
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [2ax5, stx8, fne2, ukpm, m9sd]
tasks: []
---

## The ask

The human, 2026-10-06 ~5:45 PM ET (verbatim): "We're missing gates for this. We should be writing a change spec (something like proposal and design) using well-written prompts to guide the agent for the work. The design proposal (not the ticket) should be reviewed for risk and impact. We need to stop passing off tickets to workers so quickly for implementation without design up front and review."

Earlier the same week: "The worker agents that are doing these things are just making random guesses" (stx8), and "I don't want a design document. I want a ticket. The ticket is, for now, the design document" (fne2/ukpm). Settle how those fit.

## The problem

A ticket goes to the PM, the PM writes a brief, and a worker builds. Nothing in between proposes the change, designs it, or checks it for risk. Incident 2ax5 (br-3haz broke cross-project messaging for ~22 h) is the result: a breaking wire change with no migration path, no staged rollout and no check against running daemons. docs/design/gates.md says human approval is "by role prompt and convention, not a mechanism", and every configurable gate there is Planned.

## What to design (design first; this ticket goes to the designer role, br-ukpm, once the human has reviewed it)

- **A change spec** between ticket and build: a proposal (why, what changes, what doesn't) and a design (exact names, states, wire and CLI changes, migration, rollout, fallback), written by an agent guided by well-written prompts.
- **A risk and impact review of the proposal and design** (not the ticket) before any worker starts: what breaks for running daemons, other machines, other projects, stored data; how it rolls out and rolls back. Who reviews (the human, a reviewer agent, both) and when the human must.
- **A real gate:** a task can't be planned for building until its change spec is approved. Enforced by the daemon or CLI, not by a role prompt. Which changes need it (wire format, schema, CLI behaviour, messaging, upgrade) and which skip it (docs, small fixes).
- **Where it lives:** in the ticket, or a separate artifact (the human mentioned OpenSpec-style proposal and design). Relate it to docs/design/spec-flow.md and specs.md (built, not wired in) and gates.md (planned).
- **Prompts:** the prompts for proposal, design and review are part of the deliverable.

Related: 2ax5, stx8, fne2, ukpm, m9sd, gates.md.
