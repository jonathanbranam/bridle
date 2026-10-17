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

8. **Templates and instructions per stage, from bridle rules** (the human, 2026-10-16 ~10:58 PM
   ET: "Everyone of these must have a fixed template and then instructions on how to write them.
   Whatever agent does the work gets that template and instructions. And those should be built
   from bridle rules. This is how that system gets to an agent"). Each stage has a fixed template
   and writing instructions in the workflow (`workflow/base/`, beside rules and roles), and the
   agent writing a stage is given both. Related: br-86c6 (the workflow doesn't reach agents).
9. **A proposal, agreed with the human, before ready** (the human: "a ticket should have a
   proposal worked out with human. A ticket without a proposal isn't a ticket ready to be
   worked"; "PM writes proposal"; "The proposal can live in the ticket yes but it has to follow
   the template"). The PM writes it with the human; `ticket ready` needs it. For small kinds it
   may be the ticket body, in the proposal template.

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
told, an agent writes. Who writes (the human, 2026-10-16: "PM writes proposal"): proposal = project
manager, with the human (see Q5); design and specs = designer; tasks = project manager. The
designer's `## Design options` moves from the ticket body to `design.md` (the body keeps the ask).

Stage contents, one line each: proposal = why, what changes, what does not; design = options and a
recommendation (this role's output today); specs = the requirements the change adds or alters, in
bridle spec syntax; tasks = the worker briefs, in order, each small enough for one worker.

Which tickets get stages: the ask says every ticket, but a typo fix does not need four files
(cost-of-not-doing). Recommend a ticket field `stages` defaulting by kind (feature, arch-revision,
epic: all four; fix, docs, chore: none; `bridle ticket set <id> stages ...` to override). Settled by
the human (2026-10-16, "Sure. It doesn't matter that much we can change it later"). The default
lives in one table in the code or config, so changing it later is a one-line edit.

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

A + A + A + A, plus Q5 (A for stage kinds, B for small kinds) and Q6-A: folder beside the ticket; `review:`/`approved:` fields with `bridle ticket approve`;
`bridle ticket stages` plus one daemon notice to the PM; specs.md as a draft merged by the first
task; `ticket ready` requires an approved proposal; one rule per stage holds its template and
instructions and reaches the writing role through the existing rules path. Accepted costs: approval is not invalidated when an approved file is edited; the daemon never
spawns agents itself; spec draft can drift until merged; markdown-only attachments for now.
Smallest first slice that proves it: folder + fields + `stages` command, the designer writing
`design.md`, then the UI tabs (the epic's "one real ticket end to end").

### Q5. The proposal is agreed with the human before the ticket is ready (item 8)

The human, 2026-10-16: "a ticket should have a proposal worked out with human. A ticket without a
proposal isn't a ticket ready to be worked." And: "The proposal can live in the ticket yes but it
has to follow the template."

- **A. The proposal is a stage like the others, and `ticket ready` requires it.** For a ticket with
  stages, `bridle ticket ready` refuses unless `proposal` is written and in `approved:`. So the
  proposal is always a reviewed stage: for these kinds `review:` implicitly contains `proposal`
  (the human can still drop the other stages from review). The PM drafts it, talks it through with
  the human (the thread, or the Proposal tab with highlight-to-comment), then `ticket approve
  <id> proposal`. Cost: `ready` learns one more check, next to its existing "empty ask" and
  "uncommitted" refusals. Falls short: none for stage kinds.
- **B. Small kinds (fix, docs, chore) have no stage files; the proposal lives in the ticket body.**
  The body must follow the proposal template (headings below). `ticket ready` for these kinds
  checks the template headings are present and non-empty (the PM writes them with the human), and
  the human's ready/approve is the agreement. Same rule for everyone: no proposal, not ready. Cost:
  a heading check in `ready`. This answers the thread's open point: the ask alone is not the
  proposal, but a three-line body in the template is enough.
- **C. Only the human's `ready` is the agreement** (today). No mechanism; a ticket can be ready with
  nothing agreed. This is the failure the human described. Rejected.

Recommend **A for stage kinds and B for small kinds**: one rule ("not ready without an agreed
proposal"), two places the proposal can live. For B the template is deliberately short.

### Q6. Templates and writing instructions come from bridle rules (item 9)

The human, 2026-10-16: "Everyone of these must have a fixed template and then instructions on how to
write them. Whatever agent does the work gets that template and instructions. And those should be
built from bridle rules. This is how that system gets to an agent."

What exists: rules are files in `workflow/base/rules/<id>.md` (frontmatter `id`, `severity`,
`roles`), resolved over base, packs and the project, and delivered in a spawned agent's system prompt
for its role (workflow-layers.md: built and in use). `bridle rules explain` shows the result. Roles
already say which rules they get via `roles:`. So the delivery path exists; the gap (br-86c6) is
that nothing in the rules says how to write a proposal, design, spec or task list.

- **A. One rule per stage, delivered to the writing role.** `workflow/base/rules/stage-proposal.md`
  (`roles: [project-manager]`), `stage-design.md` and `stage-specs.md` (`roles: [designer]`),
  `stage-tasks.md` (`roles: [project-manager]`). Each rule body has two parts under fixed headings:
  `## Template` (the exact headings the file must have, with a one-line prompt under each) and
  `## How to write it` (the instructions). A project overrides a stage by shadowing the rule in
  `.bridle/rules/` (layering already does this). Cost: four rule files; the writer's prompt grows by
  about one page each, only for the roles that write that stage. Falls short: an agent has all of its
  role's templates in its prompt even when writing one stage; a long-lived PM carries two.
- **B. The unblock notice carries the rule.** Same four rule files, but delivered on demand: the
  Q3 notice names the stage, and `bridle rules explain stage-<name>` (or `bridle ticket stages <id>
  --template`) prints the template and instructions for the agent to read when it starts the stage.
  Smaller prompts, but depends on the agent running the command: the same "convention, not
  mechanism" gap as br-86c6, and workers do not run `prime` today.
- **C. The template as a file the daemon copies into the ticket folder** (`design.md` created with
  the headings filled in). Gives the human a visible skeleton and makes "follows the template"
  checkable, but the instructions still need a delivery path, so it is A or B plus this. Add only if
  A's check (below) is not enough.
- **D. Templates in the role prompts** (`workflow/base/roles/*.md`). Not overridable per rule, mixed
  with the role's other text, and the human asked for rules. Rejected.

Recommend **A**, plus a light check: `bridle ticket check` verifies a stage file has the template's
headings (parsed from the rule's `## Template` block), so "follows the template" is enforced, not
asked. B is the fallback if prompt size becomes a problem. This is the first real use of the rules
path to an agent for an artifact, and it closes the "writes the stage but invents the format" gap
that br-95mu's "well-written prompts" asked for.

Proposal template headings (so the human can react to them now): Why; What changes; What does not
change; Risk and impact; Out of scope; Open questions. Design: Problem; What exists today; Options
(each with user view, internals, cost, shortfall); Recommendation; Rejected. Specs: one block per
capability in spec syntax, as in Q4. Tasks: ordered checklist, each item one worker brief with its
check. These are the first draft of the rule bodies, to be reviewed as the rules, not now.

### Open questions for the human

1. Q3 (unchanged, for the morning): does editing an approved stage un-approve it now (option 2C), or
   only when it bites?
2. For small kinds (fix, docs, chore), is a short template-shaped body enough as the agreed proposal
   (Q5-B)?
3. Rule delivery: all of a role's stage rules in its prompt (Q6-A), or on demand (Q6-B)?
