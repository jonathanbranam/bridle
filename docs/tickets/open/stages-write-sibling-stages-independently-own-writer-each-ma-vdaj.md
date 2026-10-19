---
id: br-vdaj
title: "Stages: write sibling stages independently (own writer each, maybe in parallel), so design and specs don't shape each other"
kind: feature
created: 2026-10-19
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-t6ev, br-2ae8, br-ewk8, br-tj7q, br-upxq]
---

## The ask

Follow-up for v1 to br-t6ev (the easy fix: one designer writes every unblocked stage in turn). Found in the human's hands-on try (br-ewk8, ticket br-2ae8).

The human, 2026-10-19, to advisor tj7q:

> Specs and design should be independent. The specs should not depend on the design, and vice versa. We don't necessarily want things the agent thought about for the design to affect the specs and the way they're written, and similarly, vice versa. Although in some sense, the spec should be more independent, writing the specs should depend on reading the existing specs as well.

> If they run in parallel, then they could run on separate agents and both get done together. That doesn't seem super important in terms of speed. The agents are the fast part, it feels like, in most cases.

> File something that we can follow up with for V1. The branching, like every stage that can be done, should be done. However that works is a little TBD.

The ask: every stage that can be written is written, each independently of its siblings (a writer of one stage doesn't carry what it thought for another; for specs, it reads the existing specs). Likely one writer per unblocked stage, on its own branch, in parallel. Open, for the design: how parallel stage branches land on main (stage-land per branch, br-upxq), how a redo (`changes`) of one stage leaves the other alone, and the writer cap. Bring the design to the human; don't ready it without them.
