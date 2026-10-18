# Stages

Status: built and in use as each step of epic br-tj7q lands. Step 1 (br-9mqh) is built: the
schema, the proposal section, the stage folder, `check`, `resolve` and `show`. Step 2 (br-quvh) is
built: the stage rules, templates and `ticket next`. Step 3 (br-ussh) is built: `stage-done`,
`approve`, `changes`, `restage` and ready-means-prepared. Step 4 (br-ewhy) is built: the daemon
spawns the designers and runs the timer (below). Step 5 (br-6c8x) is built: the worker rule and
the land check (below).

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

## Instructions and templates

Each stage has a rule `stage-<stage>` in `workflow/base/rules/` (tagged `roles: [stage-writer]`,
a role no agent has, so it reaches a writer only through `ticket next`) and a template
`stage-<stage>.template` beside it. Both layer like any rule: base, packs, project
(`.bridle/rules/`); the last layer with a template file wins, and a rule is overridden with the
usual `override:`. The `.template` extension keeps templates out of the rule loader.

`bridle ticket next <id>` picks the first unblocked stage (schema order) and prints its rule
body, its template, the output path (the ticket file for the proposal, else
`<folder>/<stage>.md`) and the files of the stages it requires. A stage waiting on an unsigned
gate is refused (exit 1, naming the stage); with nothing left it says "prepared". `--comments`
adds the human's thread comments newer than the output file's mtime (the ticket file's, for the
proposal), for a redo. `--json` is for bridle's own use.

## Writers and the timer (br-ewhy)

The daemon's stage watch (`stage_watch.rs`, on the 30 s settle-wake tick) reads the version 2
tickets in `docs/tickets/open/` of the project checkout. For a ticket whose row is `pending` and
that is not prepared:

- **Unclaimed** means no `claimed_by:` line in the frontmatter (the human's signature claim and
  the filer's `--claim` both write it) and no row claim. A claimed ticket gets no writer.
- With an unblocked stage and no writer running, it spawns a `designer` through the supervisor
  (so the budget governor, load hold, drain and settle hold apply; it also waits while as many
  designers run as `max_workers`). The prompt names the ticket and stage and tells the designer to
  run `ticket next <id> --json --comments`; comments mean a redo after `changes`, which the prompt
  asks it to address. The agent is `designer-<id>-<stage>` (a generated name for a redo). The
  designer writes the stage, runs `stage-done`, commits on its branch and tells its manager; a
  stage counts as done for the daemon when it is written on the project checkout, so the
  branch must land first. Then the next tick finds the next unblocked stage, a signature wait
  (`claimed_by: human`), or a prepared ticket.
- A stage not written `[stages] timeout` (default 60m, `0` turns the watch off) after its writer
  started, or whose writer ended without `stage-done` (not even in its worktree), adds a thread
  note and sends one message to `external:advisor/product-manager`: "br-oooo: design not done by
  designer-3, timed out". It is not retried. The product manager may `ticket restage`, or any
  later edit of the ticket file lets the next tick start a writer.
- The writers are tracked in memory; after a restart a running `designer-<id>-<stage>` agent is
  taken back by its name.

## The worker and the land check (br-6c8x)

The rule `stage-apply` (`roles: [worker]`, copied from OpenSpec's apply skill) tells a worker on
a version 2 ticket to read the proposal, `design.md`, `specs.md` and `tasks.md`, merge specs.md
into `design/specs/` as the first task, work through `tasks.md` in order ticking each item
(`- [x]`), and pause on a blocker. Version 1 workers ignore it. `integrator::land` refuses a
version 2 ticket whose landed `tasks.md` has `- [ ]` items (`Refused`, naming the first); version 1
tickets, and a version 2 ticket with no `tasks.md`, are not checked. Rules injected into the apply
step are V2.

Code: `crates/bridle-daemon/src/stages.rs` and `stage_watch.rs`; in `crates/bridle/src/`, `stage_next.rs`; `stage_cmds.rs` has the four writing commands; `ticket.rs` has `check`, `create`, `resolve` and `show`.
