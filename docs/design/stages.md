# Stages

Status: built and in use as each step of epic br-tj7q lands. Step 1 (br-9mqh) is built: the
schema, the proposal section, the stage folder, `check`, `resolve` and `show`. Writing stages,
signing and spawning designers are later steps and are not built yet.

A version 2 ticket prepares itself in stages before a worker builds it. Version 1 tickets (no
`version:`, or `version: 1`) have no stages and behave as before; version 2 is opt-in
(`ticket new --version 2`) so a bug here cannot stop ordinary tickets.

## Schema

```
version: 2
stages:
  proposal: {review: human}
  design:   {requires: [proposal]}
  specs:    {requires: [proposal], review: human}
  tasks:    {requires: [design, specs]}
```

- Stage names: `proposal`, `design`, `specs`, `tasks`. `proposal` is always listed.
- `requires` names listed stages only and may not form a cycle. `ticket new` fills it from
  OpenSpec's graph (design and specs require the proposal; tasks require design and specs),
  keeping only the stages listed.
- `review: human` makes the stage need the human's signature.
- The indented lines are a block: the frontmatter reader skips them, so a stage named `specs`
  does not read as the ticket's `specs:` field.

## Where each stage lives

- **proposal**: in the ticket body, between two `---` lines, opening with `## Proposal` and the
  template's headings. `stages::find_proposal` finds its start and end; headings are not validated.
- **design, specs, tasks**: files in a folder beside the ticket,
  `docs/tickets/open/<slug>-<id>/design.md` (and `specs.md`, `tasks.md`). Any other file there is
  an attachment. `ticket resolve` moves the folder with the ticket file.

## Stage states

`ticket show` prints one per stage:

| state | meaning |
|---|---|
| waiting | a required stage is not yet written (and signed, if reviewed) |
| unblocked | every required stage is done; this one is not written |
| written | written, needs no signature |
| waiting for signature | written, `review: human`, not signed |
| signed | written and signed |

A required stage counts as done when it is written without review, or signed. A signature is a
`signed:` line in the stage file's frontmatter (the proposal: a line inside its section); the
signing step may change this.

## Who does what (RACI)

- **Writer** (responsible): the designer writes each stage; for the proposal, the filer
  (`ticket new --claim`).
- **Signer** (accountable): the human signs a `review: human` stage; nothing after it goes
  forward until then.
- **Watchers** (informed): agents and people following the ticket.

Code: `crates/bridle/src/stages.rs`; `ticket.rs` has `check`, `create`, `resolve` and `show`.
