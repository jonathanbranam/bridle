---
id: br-9mqh
title: "Stages MVP 1: ticket version 2, the stages schema, the proposal section and the stage folder"
kind: feature
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-tj7q
theme: product-process
---

## The ask

Step 1 of 7 of the MVP of epic br-tj7q (Tickets prepare themselves). The decisions are settled in
the epic's "Captured intent" (the human, 2026-10-17, grilled with advisor tj7q). The human: "At the
end of the MVP, we should be able to create version 1 and version 2 tickets ... I don't want every
ticket to have to be version 2 because we'll end up with a chicken-and-the-egg problem."

---

## Proposal

### Why

Stages need a place on the ticket before anything can write or gate them, and the new workflow must
be opt-in so a bug in it cannot stop today's tickets.

### What changes

- Ticket frontmatter `version` (absent = 1). `bridle ticket new --version 2` writes `version: 2`;
  the default stays 1 (step 7 flips it).
- Frontmatter `stages:` on a version 2 ticket, as approved by the human (2:55 PM ET):

  ```
  stages:
    proposal: {review: human}
    design:   {requires: [proposal]}
    specs:    {requires: [proposal], review: human}
    tasks:    {requires: [design, specs]}
  ```

  `ticket new --version 2 --stages design,specs,tasks --review proposal,specs [--claim]` writes it,
  filling `requires` from OpenSpec's graph (proposal: none; design, specs: proposal; tasks: design
  and specs, or whichever of them are listed). The proposal is always included. `--claim` claims
  the ticket for the filer, who writes the proposal.
- The proposal lives in the body: a `---` line, `## Proposal` with the template's headings, a `---`
  line. One parser finds it (start and end) for later steps.
- Stage files live in a folder beside the ticket: `docs/tickets/open/<slug>-<id>/design.md`,
  `specs.md`, `tasks.md` (any other file there is an attachment). `ticket resolve` moves the folder
  with the file.
- `bridle ticket check`: version 2 needs a well-formed `stages:` (known stage names, `requires`
  only on listed stages, no cycles); version 1 is unchanged. The daemon indexes the new fields;
  `ticket show` prints each stage's state (waiting, unblocked, written, waiting for signature,
  signed).
- A new `docs/design/stages.md` (Status: built and in use as each step lands): the model, the
  schema, the stage states, and who does what in RACI words (writer responsible, signer
  accountable, watchers informed); `cli.md` and `storage.md` in step.

### What does not change

Version 1 tickets: their fields, check, states and every command behave exactly as today.

### Risk and impact

Touches `ticket new`, `check`, `resolve` and the daemon's ticket index, which every ticket uses.
Tests must show a version 1 ticket round-trips unchanged.

### Out of scope

Writing stages, signing, spawning (steps 2-4). Validating the proposal's headings.

### Open questions

None.

---
