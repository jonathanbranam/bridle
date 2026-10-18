# Stages

Status: built and in use as each step of epic br-tj7q lands. Step 1 (br-9mqh) is built: the
schema, the proposal section, the stage folder, `check`, `resolve` and `show`. Step 3 (br-ussh) is
built: `stage-done`, `approve`, `changes`, `restage` and ready-means-prepared. Spawning designers
and the timer are later steps and are not built yet.

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

A required stage counts as done when it is written without review, or signed.

## Marks, signing and claims

The ticket's frontmatter records the marks (the files only hold the content):

- `written: [..]`: set by `ticket stage-done` (which checks the file or proposal section is there
  and non-empty), cleared by `changes` and `restage`.
- `signed: [..]`: set by `ticket approve`, for a written `review: human` stage. The signature is
  the human's; the thread note says who ran it ("signed by human at <time>", or "signed by human
  via <principal>" when the token is an agent's) and when. Editing an approved stage does not
  withdraw its signature; only `changes` does. Cryptographic signing is a later ticket (v2va).
- `claimed_by: human` while any stage waits for the human's signature; `stage-done` ends the
  writer's claim and `approve`, `changes` and `restage` recompute it. This is a frontmatter line,
  not a row claim: a version 2 ticket stays `pending` while it is prepared and the row claim
  needs `planned`.
- `prepared: true`: written whenever every stage is written and every gate signed, dropped
  otherwise. The daemon's ticket sync (`ticket_links.rs`) moves a `pending` row of a version 2
  ticket with it to `open` itself (actor `bridle`, with a thread note). `ticket ready` refuses a
  version 2 ticket without it; `plan` and the rest are as for any ticket.

Every command adds a thread note (stage written, waiting for signature, signed, sent back, put
back), and a note reaches the ticket's watchers. A timed-out stage is step 4.

## Who does what (RACI)

- **Writer** (responsible): the designer writes each stage; for the proposal, the filer
  (`ticket new --claim`).
- **Signer** (accountable): the human signs a `review: human` stage; nothing after it goes
  forward until then.
- **Watchers** (informed): agents and people following the ticket.

Code: `crates/bridle/src/stages.rs`; `stage_cmds.rs` has the four commands; `ticket.rs` has `check`, `create`, `resolve` and `show`.
