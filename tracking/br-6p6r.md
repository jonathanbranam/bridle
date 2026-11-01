+++
id = "br-6p6r"
title = "Why project role files drift from and override base instructions, and whether rules are applied and used well"
kind = "research"
state = "pending"
created_at = "2026-11-01T13:08:02.421Z"
updated_at = "2026-11-01T13:08:28.521434Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "6p6r"
+++

docs/tickets/open/why-project-role-files-drift-from-and-override-base-instruct-6p6r.md

submitted by external:orchestrator

Relayed by orchestrator@vesta (m-0564 on dalek-home), from the human via aide on notes (notes m-0143, 2026-11-01 8:06 AM).

The human, verbatim: "Yes, fix that, and this should come from the overall system instructions or the overall bridle instructions. I guess we're overriding that. I feel like the rules system we've built doesn't actually get work properly or get used properly."

## What prompted it

The notes project's `.bridle/roles/aide.md` hard-coded `external:orchestrator@vesta`, which isn't a recipient on the notes daemon. That overrode the correct form that base `aide.md` already gives (`--project <host>-home`). orchestrator@vesta is fixing the notes file itself; this ticket is about the pattern.

## The ask (for the human to scope)

- Why do per-project role files drift from base instructions and end up contradicting them?
- Are rules actually applied and used well by agents?
- Possible directions to evaluate: a check that flags per-project role text contradicting base, or addresses in role files that aren't valid recipients.

Priority: normal (orchestrator@vesta's recommendation). Kind research: needs the human's scoping before it is readied.

## Thread

### note · external:orchestrator · 2026-11-01T13:08:02.421Z
submitted by external:orchestrator

### question · external:orchestrator · 2026-11-01T13:08:28.521Z
Scoping needed before this research ticket is readied. The ask: why per-project role files drift from and override base instructions, and whether rules are applied well (prompted by notes' aide.md hard-coding an address that isn't a recipient). Recommendation (pm-1, and I agree): start with one cheap, concrete step - a bridle doctor check that flags project role text naming an address that is not a valid recipient - and keep the broader 'are rules used well' research for after. Reply 'go with the check' to ready that, or say how you want it scoped.
