---
id: ukpm
title: "A designer role: a background agent that reads a problem ticket, analyses the system and writes design options into the ticket"
kind: feature
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [fne2, m9sd, 67qw, 6yb4, vp9e]
tasks: []
---

## The ask

The human, 2026-10-05 ~9:25 PM ET (verbatim):

"And we don't have a role for designer. It's kind of like a product manager, but I think this is a little more technical. These two might be similar, but what I'm looking for is an agent that's just going to be a background agent, I think. It's very detailed and very focused. I want to assert some very strong design principles that I believe in, and then we give it an intro, like an existing ticket with the problem described. Their job would be to read the ticket, analyze the current system, and then propose a solution. That proposal goes back into the ticket, I think, or in a related ticket of a different type, right? Maybe a design ticket is a ticket. I'm not sure. I just don't know yet, but it should analyze, come up with a proposal, come up with several options, probably, and really push into the design principles. Keep it simple. You aren't going to need it. I would talk about other things about modularity. Some of this is in an API design, like we're talking about here, and some of this would be in internal designs. Those would probably be different agents. They would have different prompts, potentially. I'm not sure exactly. Could be, could just be one, maybe, but I might actually spin up two separate agents for the two sides of that. It's kind of related to architecture as well: software architecture, not really systems architecture, but I think we have a lot of software architecture gaps in this project."

"Yeah, I think it's safe to work on that right away. Get started on a new role that we can use, and I'm going to want to look through the rules for that. Once that's built, then we can use it for this task."

"I don't want a design document. I want a ticket. The ticket is, for now, the design document."

## What to build

A base-workflow role, `designer` (workflow/base/roles/designer.md), for a daemon-spawned background agent, plus any rules it needs:

- **Input:** one existing ticket that describes a problem.
- **Work:** read the ticket, analyse the current system (code, CLI, docs), then propose a solution: several options, a recommendation, and the trade-offs, measured against stated design principles.
- **Output:** the proposal written into the ticket (the ticket is the design document for now), not a design doc. It does not build anything. Its task ends when the proposal is in the ticket and committed.
- **Design principles,** asserted strongly in the role: keep it simple (kiss.md), you aren't going to need it (yagni.md), modularity, one name per action and no near-duplicate commands (ticket fne2), a design read from the user's side first. The human will add their own, so leave a clear place for them.
- **Two focuses:** API and CLI design (the surface users and agents see) and internal software architecture (modules, boundaries). Decide whether that's one role with a focus flag or two roles; the human leans toward possibly two agents with different prompts. Propose, don't over-build.
- Relationship to project-manager and prototyper: say how they differ.

The human reviews the role and its rules before it's used. The first job after that is ticket fne2.

Related: fne2 (first job), m9sd, 67qw (improve the architecture of the base system), 6yb4 (the prototyper role, a model for a new base role), vp9e (are roles and rules the same thing).
