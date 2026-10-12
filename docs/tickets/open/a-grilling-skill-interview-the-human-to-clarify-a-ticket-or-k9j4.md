---
id: br-k9j4
title: "A grilling skill: interview the human to clarify a ticket or an epic, and check children against their epic's design"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
theme: product-process
---

## The ask

The human, 2026-10-12 ~12:13 AM ET, verbatim (to advisor product-manager):

> Also on the design side we need a skill
> Like grill-me (search for Matt Pocock) that we can use to clarify a ticket and esp a larger work item like an epic. This will help drill down into the details.
>
> The agent should clarify the write up and ask so it is clear for the overall design and expectations. Tickets should then be compared against the overall design of the epic (also a ticket) to clarify and understand my meaning.
>
> OpenSpec skills do some of this as well but less interactively.

## Prior art (advisor, 2026-10-12)

- **Matt Pocock's grilling skills** (github.com/mattpocock/skills, MIT). `grill-me` and `grill-with-docs` are one-line wrappers around `grilling`, an interview loop:
  - The subject is a **design tree**: decisions with decisions hanging off them.
  - Each **round** asks the whole **frontier**: every decision whose prerequisites are settled, nothing that depends on an open answer. Questions are numbered, each followed by the agent's recommended answer, so the human can answer "1 yes, 2 the second, 3 no because ...".
  - **Facts vs decisions**: facts the code or docs can settle, the agent looks up (subagents) instead of asking. Decisions are the human's; an agent that answers its own decisions has broken the skill.
  - It ends only when the human confirms a shared understanding, not when the questions run out; it does not build.
  - `grill-with-docs` also writes a glossary and ADRs as it goes. `wayfinder` is the epic-sized one: a **map** ticket (destination, decisions so far, "not yet specified" fog, out of scope) and child **decision tickets**, resolved one at a time until nothing is left to decide; then it hands off to spec and tickets.
- **OpenSpec** (Fission-AI): propose -> proposal, specs, design, tasks; living specs plus change deltas. More of a document pipeline, less interactive (the human's point).
- The c2 wiki (rule c2-wiki-in-searches): AskTheCustomer and OnsiteCustomer were reached but returned only cross-references, nothing usable; not searched further.

## Shape (sketch, for the design)

A bridle skill (`workflow/base/skills/`), run by the PdM or the designer with the human, on one ticket:

1. **Clarify the write-up.** Read the ticket (and the code and docs it names), then interview the human in frontier rounds, grilling-style: facts looked up, decisions asked, a recommendation on each question. The answers go into the ticket (the human's words quoted, rule talk-on-the-task), not just the chat.
2. **Against the epic.** For a child ticket, read the parent epic's outcome, done-when and design first. Each gap or contradiction between the child and the epic becomes a question ("the epic says X, this ticket assumes Y: which?"). The goal is the human's meaning, not the agent's reading.
3. **For an epic,** the result is a clear outcome, done-when and design (and, wayfinder-like, a list of what is not yet specified), so its children can be checked against it.
4. **Done** when the human says the understanding is shared; the skill never readies or builds.

Open for the design: one skill or two (ticket vs epic); round-based or one question at a time (the human's preference); whether it adopts grilling's text (MIT, with credit) or writes its own; how it fits the design-before-build gate (95mu) and multi-part tickets (attachments).
