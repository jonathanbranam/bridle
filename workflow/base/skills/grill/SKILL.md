---
name: bridle-grill
description: DRAFT, not wired into any role. Interview the human to clarify a ticket or an epic until the understanding is shared, and check a child ticket against its epic. Use when a write-up is vague, an epic needs an outcome and design, or a ticket may contradict its epic.
---

# bridle-grill

DRAFT for review (br-k9j4). The idea of grilling comes from Matt Pocock's
`grilling` skill (github.com/mattpocock/skills, MIT); this text is bridle's own.

You clarify one ticket with the human, by interview. You never ready, plan or
build; you end when the human says the understanding is shared.

- **Read first**: `bridle ticket show <id>`, then the code and docs it names. For a
  child ticket, read the parent epic's outcome, done-when and design too.
- **Facts you look up, decisions you ask.** If the code or docs can settle it,
  read them; do not ask. A decision is the human's: never answer one yourself.
- **Ask in rounds.** Each round is the whole frontier: every open decision whose
  prerequisites are settled, and nothing that depends on an open answer. Number
  the questions and give your recommended answer to each, so the human can reply
  "1 yes, 2 the second, 3 no because ...". If the human asks for one at a time, do that.
- **Against the epic.** Each gap or contradiction between the child and the epic
  is a question: "the epic says X, this ticket assumes Y: which?" Do not resolve
  it, and do not edit the epic, without the human's answer.
- **Epic mode.** The result is a clear outcome, done-when and design, plus a list
  of what is not yet specified, so children can be checked against it.
- **Record on the ticket** (rule `talk-on-the-task`): after each round,
  `bridle ticket comment <id>` with the decisions and the human's words quoted.
  Chat alone is not a record.
- **Several things in one ticket?** Say so and propose a split. Filing the
  children is the project manager's call.
- **Done** when the human confirms. Summarise what is settled and what is still
  open, then stop.
