---
id: br-tj7q
title: "Change stages on every ticket: attachments, proposal/design/specs/tasks files, per-ticket stage reviews, specs become bridle specs"
kind: epic
created: 2026-10-17
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [ui-fwsx]
theme: product-process
---

## The ask

The human, 2026-10-16 ~10:25 PM ET, to advisor (product-manager), verbatim:

> We need this epic done whatever it is called or the ladder:
>
> - attachments
> - change spec files on every ticket: proposal, design, specs, tasks
> - i can ask for review of specific stages per ticket or not. No other system yet. We can list
>   the stages per ticket that human wants to review.
> - agents wrote the files for all stages that are unblocked
> - specs become bridle specs when ticket is implemented

## What this means (advisor's reading; the designer confirms or corrects)

1. **Attachments.** A ticket can carry files besides its body (the attachments epic's "several
   documents on one ticket"; the human's MIME-parts idea). Images later.
2. **Four stage files on every ticket:** proposal, design, specs, tasks (the OpenSpec change
   shape). Each is a file attached to the ticket.
3. **Review per stage, chosen per ticket.** The ticket lists the stages the human wants to review
   (for example `review: [design, specs]`, or none). A listed stage waits for the human's
   approval before the next stage starts; an unlisted stage doesn't wait. No other review system
   in this epic (v2va's signed reviews stay separate).
4. **Agents write every unblocked stage.** A stage is unblocked when the stage before it is
   written and, if listed for review, approved. Agents (designer for proposal/design/specs,
   project manager for tasks) write each unblocked stage without being asked again.
5. **Specs become bridle specs on landing.** When the ticket is implemented, its specs stage
   becomes part of the project's bridle specs (`design/specs/`, docs/design/specs.md and
   spec-flow.md), so the specs outlive the ticket.

## UI (the human, 2026-10-16)

The human, ~10:30 PM ET, verbatim: "I want UI as part of this. I want attachments as tabs or
document picker inside a ticket."

6. **The ticket page in bridle-ui shows its attachments**, the stage files among them, as tabs or
   a document picker inside the ticket (the designer proposes which, or both). The human reads,
   comments on and approves a stage there. bridle-ui work is part of this epic (child tickets in
   bridle-ui).

7. **One ticket page: task and ticket combined** (bridle-ui ui-fwsx, now part of this epic).
   The human, ~10:30 PM ET: "where is the ticket to combine task and ticket in the UI? File that
   here as well" and "Put that in this epic as well. It's several clicks every time now". The
   ticket page shows the document, state and thread together (what lives on the task page
   today), with the attachments as tabs or a picker; no separate task page to click through to.

## Supersedes or absorbs

- The scheduler ladder (roadmap, 2026-10-09), steps 2-8: this epic is the pipeline the ladder
  was meant to prove. The rest of the scheduler (yfv5) becomes a ticket run through it.
- Epic `attachments` (roadmap epic 6): folded in as item 1.
- Gates epic (reviews-enforced): br-95mu (proposal and design approved before planning) is item
  3 for the stages the human lists; the v2va review slices stay separate.

## Done when

A new ticket gets the four stage files; the human can list the stages to review; agents write
each stage as it unblocks and wait only on the listed reviews; when the ticket lands its specs
are in the project's bridle specs; the ticket page shows the attachments as tabs or a picker. Proven on one real ticket end to end.

## First step

A designer pass: '## Design options' in this ticket (where stage files live: ticket folder,
state branch or attachments; how a stage is marked approved; how agents are told a stage
unblocked; how specs map into `design/specs/`), for the human's review. No build before that.

## Design options

Focus: both. Interface (stage commands, ticket fields, the ticket page) and internal architecture
(where files live, how an unblock is noticed). I could not find ui-fwsx in this repo, so item 7 is
designed from its description here and from the gateway as it stands.

Problem in a sentence: a ticket should carry four stage files that agents write in order, the human
can pick which stages wait for their approval, and the finished specs outlive the ticket.

### What exists today

- A ticket is one file, `docs/tickets/open/<slug>-<id>.md`, plus a thread in `tickets/<id>.md` on
  `bridle/state` (storage.md). `bridle ticket set` edits listed frontmatter fields only.
- The designer already writes `## Design options` into the ticket body: that is a design stage in
  all but name. The PM writes briefs; nothing writes proposal, specs or tasks files.
- specs.md: "No deltas." A task edits `design/specs/<capability>.md` in place on its branch; the plan
  commit's spec diff is the proposal. `spec check|id` exist and are local.
- Gates (gates.md) are all Planned. br-95mu wants proposal + design approved before planning; v2va
  signed reviews are separate.
- The gateway reads and guard-writes any markdown under `docs/` (`documents.rs`: GET/PUT, search for
  the picker, highlight-to-comment in the UI). It has `tickets/{id}` detail and `tickets/{id}/{action}`.
  So "a file under docs/" is already readable, writable, searchable and commentable in the UI.

### Q1. Where stage files live

- **A. A folder beside the ticket, in git on main.** `docs/tickets/open/<slug>-<id>/proposal.md`,
  `design.md`, `specs.md`, `tasks.md`, plus any other attachment dropped in the same folder.
  `ticket resolve` moves the folder with the file. Stage files are just the four reserved names;
  an attachment is any other file. Cost: a new folder convention, `ticket new` (or the first stage
  write) makes it, `ticket resolve`/`check` learn it. Falls short: markdown only until images are
  wanted (then the gateway needs a binary read; not now).
- **B. On `bridle/state`** next to the thread. Batched commits, no PR-able history, invisible to
  `just check`, and the doc picker (reads the working tree) would not see them. Rejected.
- **C. In the daemon database as blobs.** Not diffable, not on GitHub, breaks "git is the source of
  truth". Rejected.
- **D. Do nothing:** keep stages as headings in the ticket body. Costs nothing, but gives no per-stage
  review, tab or unblock signal, which is the ask.

Recommend **A**. It reuses the document API, the picker and highlight-to-comment unchanged (principle:
modularity, no new store), and folds in the attachments epic as "any other file in the folder".

### Q2. How a stage is marked approved

Ticket frontmatter gains two list fields (set with `bridle ticket set`, like `blocked_by`):

```
review:   [design, specs]     # stages the human wants to review; empty = none
approved: [design]            # stages the human has approved
```

- **A. Two ticket fields plus `bridle ticket approve <id> <stage>`** (human principal only; the UI
  button calls it; it appends to `approved:` and writes a thread entry with the time). Rewriting an
  approved stage file is the agent's job to flag with a comment; it is not tracked. Cost: one new
  verb. Falls short: an agent can edit an approved stage and nothing notices.
- **B. Approval inside the stage file** (`approved: 2026-10-17 by human` in its own frontmatter).
  Keeps state with the file, but the human's click becomes a commit to main by the gateway, and an
  agent edit can silently keep a stale "approved". Rejected: approval is the human's decision, not
  document content.
- **C. A hash of the approved file** stored in `approved:` (`design: <blob sha>`), so any later edit
  makes the stage unapproved again. Honest but over-built for now (yagni); add it if a stale
  approval ever bites. Recorded so the next reader doesn't re-propose it unprompted.

Recommend **A**. `approve` is the one name (not `review`/`accept`); `review:` lists wanted stages.
Default `review` is empty, as the human said "or not".

### Q3. How agents learn a stage unblocked

A stage is unblocked when the stage before it exists (non-empty file) and, if it is in `review`, is in
`approved`. Order is fixed: proposal, design, specs, tasks.

- **A. A read command plus a daemon notice.** `bridle ticket stages [<id>]` prints per stage:
  `waiting | unblocked | written | in review | approved`. The daemon computes the same on ticket
  write/approve and, when a stage becomes unblocked, messages the project manager once ("br-xxxx:
  design unblocked"). The PM spawns the stage's writer with the existing `bridle agent spawn`.
  Cost: one command, one event; the daemon decides nothing about who runs.
- **B. The daemon spawns the writer itself.** Fully hands-off, but it puts the spawn policy (role,
  model, budget) inside the daemon, couples tickets to the supervisor and overlaps the manager's
  job and the budget governor. Rejected for now.
- **C. Prompts only:** the designer/PM roles say "check the previous stage". No signal; an
  approval wakes nobody. This is today's failure (br-95mu).

Recommend **A**. "Without being asked again" holds from the human's side: they approve, the PM is
told, an agent writes. Who writes: proposal, design, specs = designer; tasks = project manager. The
designer's `## Design options` moves from the ticket body to `design.md` (the body keeps the ask).

Stage contents, one line each: proposal = why, what changes, what does not; design = options and a
recommendation (this role's output today); specs = the requirements the change adds or alters, in
bridle spec syntax; tasks = the worker briefs, in order, each small enough for one worker.

Which tickets get stages: the ask says every ticket, but a typo fix does not need four files
(cost-of-not-doing). Recommend a ticket field `stages` defaulting by kind (feature, arch-revision,
epic: all four; fix, docs, chore: none; `bridle ticket set <id> stages ...` to override). Question
for the human: is that default right?

### Q4. How specs.md maps into `design/specs/`

specs.md says specs are edited in place, with no deltas, and are the only artifact that outlives a
task. Keep that.

- **A. `specs.md` is a plain draft; a worker merges it.** The file holds, per capability, a
  `## design/specs/<capability>.md` heading and the requirement blocks in spec syntax, with no ids
  for new ones and the existing id for altered ones. The first task in `tasks.md` is always "merge
  specs.md into design/specs, run `bridle spec id` and `bridle spec check --require-ids`". The
  stage file stays in the resolved ticket folder as record. No new command. Falls short: the draft and
  the merged spec can drift between approval and merge; the worker's `spec check` and the reviewer
  catch it.
- **B. The specs stage is a real edit on a branch.** The designer commits to `design/specs/` on a
  spec branch and the human reviews the diff, as specs.md describes. Truest to the model, but the
  UI's tab would need a diff view and a branch per ticket before any task exists. Rejected for now.
- **C. OpenSpec-style deltas applied by a new `bridle spec apply`.** Contradicts "No deltas" and adds
  machinery. Rejected.

Recommend **A**. "Specs become bridle specs on landing" is then true when the merge task lands; the
done check for the epic is that the merged spec is in `design/specs/`. specs.md items 2 and 3 need a
one-line update (the proposal/design/tasks files are the task record).

### UI items 6 and 7 (bridle-ui)

Ticket page, one route per ticket (replaces the task page; the old task URL redirects):

```
br-xxxx  Title                         state  priority  [actions: ready, hold, drop ...]
[ Ticket | Proposal | Design | Specs | Tasks | Thread | Attachments v ]
```

- **Tabs for the fixed set, a picker for the rest.** Stages are exactly four and the human uses them
  in order, so they are tabs, each with a badge (waiting, written, needs your review, approved).
  Attachments have no limit, and lists that grow without limit are not dropdowns (bridle-ui m2pz),
  so the last tab is the existing searchable document picker scoped to the ticket folder. Both,
  as the human allowed; neither alone fits.
- **Ticket tab = document + state + thread together**, what the task page shows today, so no click
  through (item 7). Thread is its own tab on narrow screens and a side column on wide ones.
- **Review in place.** A tab in `review:` and not yet approved shows Approve (calls `ticket approve`)
  and the highlight-to-comment that documents already have. The human can also tick which stages to
  review from the Ticket tab (calls `ticket set review`).
- **Gateway:** `tickets/{id}` detail adds `stages` (name, written, in review, approved) and
  `attachments` (paths); files are read and written through the existing documents API; one new
  action `approve`. No new store.

### Recommendation and trade-offs accepted

A + A + A + A: folder beside the ticket; `review:`/`approved:` fields with `bridle ticket approve`;
`bridle ticket stages` plus one daemon notice to the PM; specs.md as a draft merged by the first
task. Accepted costs: approval is not invalidated when an approved file is edited; the daemon never
spawns agents itself; spec draft can drift until merged; markdown-only attachments for now.
Smallest first slice that proves it: folder + fields + `stages` command, the designer writing
`design.md`, then the UI tabs (the epic's "one real ticket end to end").

### Open questions for the human

1. Default stages by kind (all four for feature/arch-revision/epic, none for small kinds): ok?
2. Is the proposal stage the designer's, or should the PM write it before design starts?
3. Does editing an approved stage need to un-approve it now (option 2C), or only when it bites?
