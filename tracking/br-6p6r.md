+++
id = "br-6p6r"
title = "Why project role files drift from and override base instructions, and whether rules are applied and used well"
kind = "research"
state = "open"
created_at = "2026-11-01T13:08:02.421Z"
updated_at = "2026-11-01T13:20:29.925216Z"
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

### note · external:aide · 2026-11-01T13:20:08.486Z
The human's scoping, 2026-11-01 ~8:00 AM ET (to aide), verbatim: "I would just do the research part first. This might not even be hard. This could be just a few messages between agents to explain to me what's going on and how this works. I'm not ready for a fix yet. I don't understand how the system works, and clearly there's something in here that I'd like to understand first." So: research only, no doctor check or other fix. The output is an explanation for the human of how project role files, base role text and rules combine (and why notes' aide.md could override base), plus what evidence there is that rules are used. Readied by aide on this scoping.

### note · external:orchestrator · 2026-11-01T13:20:29.901Z
Scope set by the human (via aide, 2026-11-01 ~8:20 AM, quote above): research only, no doctor check and no fix. Deliverable: a short explanation for the human of how project role files, base role text and rules combine (what overrides what, and how rules reach an agent), and why a project file like notes' aide.md can end up contradicting base. Keep it light; a few messages or a short doc is enough.

### note · external:orchestrator · 2026-11-01T13:20:29.925Z
br-6p6r: readied by aide on a narrowed scope (research only, explanation for the human, no doctor check). See the thread.
