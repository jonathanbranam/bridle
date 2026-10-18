---
id: br-jxk4
title: Re-evaluate how tasks get planned and ordered on every project; planning is not the orchestrator's job
kind: re-evaluate
created: 2026-10-17
created_by: external:advisor/product-manager
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-dy6a
---

## The ask

The human, 2026-10-17 ~11:15 AM ET, to advisor (product-manager), verbatim (speech-to-text):

> I think the other big problematic thing right now is how other projects besides Bridle get
> their tasks planned. There's still a huge gap there. I think we need to really reevaluate how
> tasks get planned and ordered because I don't think it's meeting the needs, and I don't think
> it's maybe too complicated, kind of my thinking.
>
> It should not be on the orchestrator, right? I know that's the choice for now because it's a
> fallback. The orchestrator can do anything and has lots of permissions, so just have the
> orchestrator do it. That is definitely not the right approach. It just needs to be somewhere
> else. I don't know where yet. I'd prefer to have the aide do it or have somebody else do it. I
> don't know. It should not be another demand on the orchestrator.

Context: only bridle has a project manager (pm-1). On other projects (bridle-ui, data-contracts,
...) the orchestrator plans as "acting PM" (workflow/base/roles/orchestrator.md:65-70). Today
bridle-ui ui-sxs3/ui-qfur were readied 10-16, never planned, bounced back to pending after 4 h,
and could not be planned until the orchestrator's tokens were re-paired.

Re-evaluate how tasks are planned and ordered, for every project: who plans (not the
orchestrator), and whether the current planning/queue model (briefs, tiers, queue, ready ->
planned) is more complex than it needs to be. First step: grill the human (skill bridle-grill),
then a design pass with options. No build before the human approves.
