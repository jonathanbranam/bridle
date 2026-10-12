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

## Design

Status: draft for the human's review (pm-1 brief, designer pass). Nothing is wired or built.
The draft skill is `workflow/base/skills/grill/SKILL.md`. The skills directory has precedent
(`manager/`, `worker/`: one `SKILL.md` per role, frontmatter `name` and `description`), so the
draft follows that layout. Those two are role loops; this one is a technique a role uses.

### 1. One skill or two (ticket vs epic)

- **One skill, two modes (recommended).** The loop is the same: read, find the frontier of open
  decisions, ask, record, repeat. Only the start differs: a child ticket first reads its parent
  epic; an epic ends with a "not yet specified" list. The draft has one short section per mode.
- Two skills (`grill`, `grill-epic`). Rejected: duplicates the loop, and a ticket that grows into
  an epic mid-interview would switch skills.
- A wayfinder-style map plus decision tickets for epics. Rejected for now (yagni): the epic's own
  file already holds outcome, done-when and design; add a map ticket only if an epic outgrows it.

### 2. Rounds or one question at a time

- **Rounds (recommended default).** Each round asks the whole frontier (every decision whose
  prerequisites are settled), numbered, each with the agent's recommended answer, so the human can
  reply "1 yes, 2 the second, 3 no because ...". Fewer round trips, and the human can skim.
- One question at a time. Kept as an opt-out: if the human says "one at a time", do that. It is
  the human's stated preference to confirm (see questions), so the skill follows whichever they
  say; it does not make it a setting.
- Rejected: a fixed question list up front. The questions after round 1 depend on the answers.

### 3. Adopt grilling's text or write our own

- **Write our own (recommended).** The idea (design tree, frontier rounds, facts looked up vs
  decisions asked, recommendation on each, ends on the human's confirmation) is not text and needs
  no licence; bridle's skills are short and use bridle's commands and rules, so a copied body
  would be mostly rewritten anyway. The draft credits the source of the idea in one line.
- Adopt `grilling` verbatim with the MIT notice. Rejected: it does not know tickets, the thread or
  our rules, and would carry text we then maintain.
- Fetch check: github.com/mattpocock/skills was reachable (raw.githubusercontent.com README
  fetched); the skill body was not read because the ticket's prior-art summary was enough.

### 4. Fit with the design-before-build gate (95mu) and multi-part tickets

- The skill is the interview that produces the written understanding; 95mu's change spec is the
  reviewed artifact. **Grilling comes before the change spec** and feeds it: its output (the
  ticket's clarified ask and the decisions recorded on the thread) is the input to the proposal.
  It is not itself a gate and never readies, plans or builds; 95mu stays the only gate.
- Where answers go: the ticket thread (`bridle ticket comment`, the human's words quoted), and
  the ticket file's body when an answer changes the ask (the designer edits the body; the thread
  keeps the quote). Chat alone is not a record (talk-on-the-task).
- Multi-part tickets: if the interview shows the ticket is several independent things, the skill
  says so and proposes a split to the human; it does not split or file children itself (filing
  is the PM's call, work-flow). Reading "attachments" as files or sub-parts hung on a ticket: the
  skill reads what the ticket names and links; if that means something else, see the questions.
- Epic check on children: the skill asks, per gap or contradiction, "the epic says X, this
  ticket assumes Y: which?". It never resolves the contradiction itself and never edits the epic
  without the human's answer.

### Verify

`bridle ticket check` clean; the draft is ASCII; a dry read by the human. No code, no tests.

### Questions for the human

1. Rounds by default, with "one at a time" on request: is that right?
2. Write our own text and only credit the idea: OK, or do you want grilling's text adopted?
3. By "multi-part tickets (attachments)" did you mean tickets with attached files, or tickets
   made of several parts? The draft handles the second by proposing a split.
4. Which role runs it first: the PdM, the designer, or either?
5. Should the skill be allowed to edit the ticket body (with your answer quoted on the thread),
   or only comment and leave the edit to the PdM?

## Decisions (the human, 2026-10-12 ~8:30 AM ET)

Verbatim (to advisor product-manager):

> Keep it simple: maybe one skill, and it says, "Ask if this is an epic or a ticket." That'd probably be, I guess, what I'd say first.
>
> I think we can take as much from Matt as we want. ... I really prefer one or two questions at a time. ... I feel like it's overwhelming to get a ton of questions that I'm supposed to answer. I'd always said that it leads me to kind of stop the work.
>
> If it's MIT licensed, we could just copy GrillMe to start with and have it available for our agents. That would be totally fine.
>
> 1. Copy GrillMe.
> 2. Attribute Matt.
> 3. Just make it work in our system. ... make it work with Bridle, so it will use bridle tickets for this, basically, and then make our version.

So this is a build, not a design pass (supersedes "Open for the design" above):

1. **Copy** Matt Pocock's `grilling` skill (the body behind `grill-me`) from github.com/mattpocock/skills, `skills/productivity/grilling/SKILL.md` at commit 95249b0b, MIT, Copyright (c) 2026 Matt Pocock, into `workflow/base/skills/grill/SKILL.md` (name `bridle-grill`, or what fits the existing `bridle-manager`/`bridle-worker` names). Keep its core: the design tree, facts looked up (subagents) and decisions asked, a recommended answer on each question worded so "yes" accepts it, and done only when the human confirms a shared understanding; it never builds.
2. **Attribute**: the MIT notice in the skill file (or a LICENSE beside it) and a credit line with the source URL and commit.
3. **One skill.** Its first question: "Is this an epic or a ticket?"
4. **One or two questions at a time**, not grilling's whole-frontier rounds. Because the human otherwise assumes a batch is all that's needed, the skill keeps the open questions written down on the ticket and says, with each question, how many are open now (the count can grow as answers open new branches).
5. **Bridle tickets** are the record: it reads the ticket and what it names; each answer goes on the ticket's thread quoted (`bridle ticket comment`), and the settled result into the ticket body (the ask, outcome, done-when, design). For a child ticket it reads the parent epic first and turns each mismatch with the epic into a question.
6. It never readies, plans or builds; it ends with "shared understanding?" and the human's yes.

Acceptance: the skill is installed for agents like the existing ones; a short note in the docs where skills are listed; `just check` green. Size S.
