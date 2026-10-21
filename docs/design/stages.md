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
- `review: human` makes the stage need the human's signature. `ticket new --review` sets it; `bridle ticket set <id> review design,specs` changes it later (the list replaces the set, empty clears it; refused for a stage the ticket does not list, for a stage already signed, and on a version 1 ticket).
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
  withdraw its signature; only `changes` does. `approve --comment/--comment-file` takes the human's
  words with the signature, as a second thread note right after it. Relayed words (that, `ticket
  comment --for-human`, and `changes` text run by an agent) keep the real principal as `from` and
  carry a `for-human` mark (a fourth part of the thread header); the daemon records the mark only
  for `external:advisor*`, `external:aide`, `external:orchestrator`, a product-manager agent, or
  the human, and refuses workers and designers. Cryptographic signing is a later ticket (v2va).
- `claimed_by: human` while a stage waits for the human's signature **and no stage is left
  unblocked** (br-t6ev; chosen over making the watch ignore the claim, because the watch then
  needs no second rule: a claimed ticket still gets no writer). So after the proposal is signed,
  design and specs are both written before the ticket reaches the human, who approves or sends
  back each on its own; `tasks` waits for both signatures. `ticket new --version 2
  --claim` writes `claimed_by: <filer>` the same way (br-r965), and `stage-done` ends the
  writer's claim and `approve`, `changes` and `restage` recompute it. This is a frontmatter line,
  not a row claim: a version 2 ticket stays `pending` while it is prepared and the row claim
  needs `planned`.
- `prepared: true`: written whenever every stage is written and every gate signed, dropped
  otherwise. The daemon's ticket sync (`ticket_links.rs`) moves a `pending` row of a version 2
  ticket with it to `open` itself (actor `bridle`, with a thread note), then runs the same effects as
  the API's ready (`OpenWatch::ready_effects`): the open-and-not-yet-planned message to the PM (else
  the orchestrator), the state-change event and the filed notice (br-h5kz). A prepared version 2
  ticket left `open` past `[tasks] open_stale` is not sent back to `pending` (nothing to
  re-prepare): the PM and the orchestrator are told again, once per period. `ticket ready` refuses a
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
adds the human's thread comments (an entry from the human, or marked `for-human`) newer than the output file's mtime (the ticket file's, for the
proposal), for a redo. `--json` is for bridle's own use.

## Landing a stage is not finishing the ticket (br-fyww)

`ticket done` (and so a land) refuses a version 2 ticket that is not prepared, read from the
ticket file on the project checkout with `stages::prepared`, the way the stage watch does. A
designer branch (`bridle/designer-*`) carries a stage, not a build. `ticket reopen` of an
unprepared version 2 ticket gives `pending` (not `reopened`), so the stage watch picks it up
again; version 1 and prepared version 2 tickets reopen to `reopened` as before.

## A stage is on main before anyone is asked (br-upxq)

The human reviews on main, not on a branch. So `stage-done` from a designer's worktree (a
`bridle/*` branch) does not just mark the file:

1. The designer commits its stage, then runs `bridle ticket stage-done <id> <stage>`. The command
   refuses uncommitted changes to the ticket or its folder, writes the marks (`written`, the
   human's `claimed_by`, `prepared`) and commits them, then asks the daemon
   (`POST /v1/tickets/{id}/stage-land`).
2. The daemon (`stage_land.rs`, under the one-landing-at-a-time lock) checks that the branch's
   changes against the integration branch touch only `docs/tickets/open/<ticket>.md` and
   `docs/tickets/open/<ticket>/`, and that the project checkout has no uncommitted edits to those
   paths, then merges the branch into the checkout. It does not touch the ticket row (a stage is
   not the build, br-fyww).
3. Only then does it add the thread note ("stage proposal written, waiting for the human's
   signature (on main at <commit>)"). The stage watch reads the stage from main as before.
4. A refusal comes back to the designer at once as the error of `stage-done`: the path outside
   the ticket, an uncommitted edit on main, or the files that conflict with main. It fixes the
   branch and runs it again. A designer that ends without a successful `stage-done` is the stage
   watch's alert below: a thread note and one message to the product manager, not the manager.

`stage-done` run outside a `bridle/*` branch (the filer's own checkout after `ticket new --claim`)
keeps the old behaviour: it marks the file in place. Rejected: designers committing straight to
main (shares the human's checkout, no isolation); a manager landing designer branches (what
failed in the first try, ticket br-2ae8).

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
  branch must land first. The one designer then runs `ticket next` again and writes the ticket's
  next unblocked stage, fully, before the one after (br-t6ev; each from its own `depends_on`
  inputs, not from its sibling), and ends when none is left. While its turn runs (starting or
  working; an idle writer has ended) it holds the ticket,
  so no second writer starts; if it ends early the next tick starts one for what is still
  unblocked. At the end the tick finds a signature wait (`claimed_by: human`) or a prepared
  ticket.
- A stage not written `[stages] timeout` (default 60m, `0` turns the watch off) after its writer
  started, or whose writer ended without a successful `stage-done`, adds a thread
  note and sends one message to `external:advisor/product-manager`: "br-oooo: design not done by
  designer-3, timed out". It is not retried. The product manager may `ticket restage`, or any
  later edit of the ticket file lets the next tick start a writer. A writer whose stage was
  written and is later sent back by `ticket changes` is dropped, not alerted (br-43ug).
- The writers are tracked in memory; after a restart a working `designer-<id>-<stage>` agent whose stage is unwritten is
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
