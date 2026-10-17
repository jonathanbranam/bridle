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

## Captured intent (2026-10-17, being grilled; supersedes the readings below where they differ)

The human's overview, 2026-10-17 ~11:30 AM ET, is on the thread verbatim. Key lines:

> When we have work to do, there should be a workflow assigned to a ticket. That workflow governs
> how the ticket is prepared to be built, investigated, researched, planned, designed, and
> implemented. [...] this workflow is not about scheduling or prioritization. It's about designing
> the ticket properly and getting approvals that are needed.

> We're just going to copy OpenSpec more or less for now. We'll change this later

> Every ticket has to have a proposal, a design, specs, and tasks before it gets handed to a
> worker. [...] the proposal always needs a human approval, and the other three are just optional.
> When we create a ticket, we'll set which ones need approval. If that's all, if we just ship that,
> we're ready.

> When we tell an agent it is time to write the proposal, we give it all the instructions for
> writing a proposal and the template right there, every single time.

### Purpose

A workflow on each ticket: the steps that prepare it for a worker, each producing a file, each
with a gate. Not scheduling, not priority.

### MVP as stated (draft; open questions below may change it)

- **The schema lives on the ticket.** It is the ticket's workflow: which stages (OpenSpec's
  artifacts) to write, which depend on which, and the gate on each. Standard schemas exist and can
  be assigned, but for now the ticket carries its own (the human, 2026-10-17: "we're simplifying
  that for the moment and just saying the schema lives on the ticket. The schema tells you which
  artifacts to create in what order and which artifacts are dependent on other artifacts").
- **Schema shape** (the human, 2:55 PM ET: "Approve"): frontmatter, one entry per stage with what
  it requires and who reviews it:

  ```
  stages:
    proposal: {review: human}
    design:   {requires: [proposal]}
    specs:    {requires: [proposal], review: human}
    tasks:    {requires: [design, specs]}
  ```

  `bridle ticket new --stages design,specs,tasks --review specs` writes it, with OpenSpec's
  dependencies filled in. The proposal is always included. A ticket with only a proposal goes to a
  worker once the proposal is written (and signed, if reviewed).
- **No rule ahead of time** on which stages a ticket has (not all four, no default by kind):
  whoever creates the ticket decides its stages and which need the human's approval (the human:
  "there's no rule. We don't need to make a rule for this ahead of time"). MVP's stages are
  chosen from proposal, design, specs, tasks. Dependencies copy OpenSpec: proposal first; specs and
  design each require the proposal; tasks require specs and design.
- Each stage produces a file (OpenSpec's "artifact"; file, attachment, artifact are one thing).
- **Where the files live** (settled 3:12 PM ET, "Yes"): a folder beside the ticket on main,
  `docs/tickets/open/<slug>-<id>/design.md`, `specs.md`, `tasks.md`, plus any other attachment;
  it moves with the ticket on resolve. The docs viewer already reads and comments on them.
- **Each stage has a gate.** MVP gate: the file exists, plus, if the stage needs approval, the
  gate's owner signs it approved. Nothing after it can go forward until it is signed. MVP owner:
  the human. Later the owner can be another agent, and it works the same way (the human: "whoever
  owns the gate, will sign it as approved, and then the rest of the command can't go forward
  unless it's been signed").
- **Every ticket has a proposal, no matter what** (a default for now, not yet a rule; the human,
  2026-10-17 ~2:30 PM ET: "every ticket needs a properly written proposal, no matter what" and
  "even a critical bug fix should have a properly written proposal for what the work is going to
  be"). Not a list of problems ending in "fix it": what the problem is, the approach and the scope,
  clear enough that a worker does not guess. Written when the ticket is created, or someone is
  due to write it then.
- **The proposal may be the ticket body**, written in the proposal template ("it's not a separate
  artifact. That's just how the ticket should be written"). Where it starts and ends must be
  marked, so it can be validated later; validation itself is not MVP ("I don't want to get too in
  the weeds on validating these templates at this point").
- **The proposal's writer gets its instructions and template** ("That's super important").
- **The proposal's gate is like any other stage's**: human review only when the ticket lists it
  (the human, 2:40 PM ET: "The gate for proposal is the same as the others. Every ticket gets one,
  but it only needs a human review if we've added that. Agents can still file bugs in critical
  things, write a proper proposal, and get to work on them"). Approval for every stage is set per
  ticket at creation.
- **Where the proposal sits**: in the ticket body, under `## Proposal` with the template's headings
  beneath it, set off by a `---` line before and after ("I'm fine with the H2, but I'd like to add
  an HR before and after it. Three dashes is plenty"). Design, specs and tasks are files.
- **One way to write a stage, for any agent.** Like OpenSpec's continue skill: the agent asks
  bridle for the ticket's next unblocked stage, and bridle prints its instructions (compiled from
  the layered bridle rules) and its template; the agent writes that one file. It works the same for
  a background agent and an interactive one (an advisor going through the ticket live with the
  human). Skill or command, the human does not mind. Instructions and template come at the moment
  of writing, every time, not in CLAUDE.md or the role prompt (overturns design Q6-A).
- **Who writes (MVP): bridle starts a background agent for every stage** (the human, 3:25 PM ET:
  "Let's just decide right now for the MVP: all the artifacts, all the stages are written by a
  background agent"; "I don't think the product manager does anything. Bridle should do this").
  The flow: when a ticket's next stage is unblocked and the ticket is unclaimed, bridle spawns an
  agent whose prompt is the ticket, the stage, the stage's instructions and its template. The
  agent writes the file and marks the stage done. Bridle sees it and works out what is next: if
  the stage needs the human's approval, the ticket is assigned to (claimed by) the human; when
  the human approves, it goes back to bridle; otherwise bridle starts the next stage's writer.
  Overturns design Q3-A (notify the PM) for Q3-B (bridle spawns). Interactive and PM-written
  stages come later.
- **The writer is the `designer` role** (settled 3:45 PM ET; the human: "that's originally what I
  thought of as the designer: to write the tickets"). Its role file widens from "a proposal into
  the ticket" to every stage, each stage's rule supplying the specifics; it still builds nothing.
  A UI designer later will be `ui-designer`.
- **Done, or not done** (settled 3:55 PM ET): the writer runs `bridle ticket stage-done <id>
  <stage>`; bridle checks the file is there and moves on. A writer that ends without it, or a
  stage not done in time, goes to the product manager with no retry ("Just send it straight back
  to the PDM if the agent doesn't finish it. No reason to try again"); the PdM can put the stage
  back for bridle ("the authority to re-ready the ticket for that stage"). Later, not MVP: the
  writer can send a stage back as blocked or needing input, with its reason on the thread, for a
  human review, and the stage instructions say so ("the proposal as written is not something we
  can implement, or the implementation is very complex").
- **Timer and redo** (settled 4:05 PM ET, "Yes"): a stage not marked done 60 minutes after its
  writer started (one setting), or whose writer ended without marking it, sends one message to
  the product manager (`external:advisor/product-manager`); the PdM runs `bridle ticket restage
  <id> <stage>` to put it back for bridle, or handles it otherwise.
- **Changes asked by the human** (settled 4:12 PM ET, "Yes."): the human comments on the thread or
  the file and says "changes" (or has an agent say it); the stage goes back to unclaimed, bridle
  starts a fresh designer whose prompt adds the human's comments, and the rewrite comes back for
  review.
- **`bridle ticket next <id>`** (settled 4:14 PM ET, "Yes"): prints the next unblocked stage, its
  instructions (the stage's rule, layered: base, packs, project), template, output path and the
  files it depends on; refuses when the stage waits on an unsigned gate. Bridle uses the same
  output as the designer's prompt, so a background and a later interactive agent get the same.
- **A signature** (settled 4:16 PM ET, "Yes"): a thread entry (signer, who ran it, time) plus the
  ticket's signed list. While a gate is unsigned, later stages do not start, `ticket next`
  refuses and the ticket cannot become ready. Cryptographic signing stays with v2va.
- **Specs become bridle specs** (settled 4:20 PM ET, "Yes"; design Q4-A): `specs.md` is a draft in
  bridle spec syntax, grouped by capability, keeping existing requirement ids on what it alters;
  the tasks template always starts with "merge specs.md into design/specs, run `bridle spec id`
  and `bridle spec check`". No new command.
- **Building from tasks.md** (settled 4:24 PM ET): one worker builds the whole ticket, following
  tasks.md as its checklist and ticking items off (OpenSpec's apply); no child rows.
- **The proposal exception: claiming.** The agent filing a ticket may write the proposal itself at
  once. A ticket waiting for a stage is claimed or unclaimed: claimed means its claimant is
  writing it, and bridle waits; unclaimed means bridle sends it straight to a background writer
  ("an unclaimed ticket waiting for a proposal should go"). Settled 4:10 PM ET: `bridle ticket new --claim`, or a
  body that already holds the proposal, keeps it with the filer; otherwise bridle starts a
  designer on it at once.
- With human review: the agent writes the file and says it is ready for review; the human
  approves, or asks for changes and then approves. Without: it goes straight on.
- **Approving**: `bridle ticket approve <id> <stage>`. The human need not type it: they say
  "approved" to whoever they are talking to (an advisor, aide, the PM), who runs it for them. Every
  approval records the signer (the human), who ran it (the human, or the agent's principal: "human
  via <agent>"; detected from the caller's token, never claimed) and the time. UI and CLI are not
  told apart (the human, 2:50 PM ET: "I want to know where that came from, just so I can be like,
  'What happened?'" and "The difference between whether I use the UI or the command line is
  unimportant"). V1 adds the button.
- **Ready means prepared** (the human, 3:05 PM ET: "Once it's been through these stages, if none
  of them require human review, the ticket should just plow through all the stages. Agents should
  fill out everything, and then the ticket's ready"; settled 3:10 PM, "Approved"). A new ticket
  is preparing: agents write each stage as it unblocks and stop only at a gate listed for the
  human. When every stage is written and every gate signed it becomes ready by itself; there is
  no separate ready approval (the human's say is the gates they list). When a ready ticket is
  built (priority, queue) is the PM's call, outside this epic ("Where this ticket fits in the
  priority and when it's actually going to be implemented is a different question"). Changes
  mhjn (who readies what); today's pending/open/planned states map onto preparing/ready, to be
  designed.

Later (named by the human, not MVP): if the merged specs differ from the reviewed specs.md and
the specs stage had a review gate, the new specs go back to that reviewer, human or agent ("TBD if
that is only an inform or an approval"; 4:20 PM ET, "far future concern but important to follow
up"); the apply step gets its own layered rules injected at the moment of work, copying
OpenSpec's apply skill freely ("having rules injected right then is really helpful for certain
very tight programming things where we want the code written in a specific way. That should be
an expectation"); gates owned by agents (other kinds of review); standard
schemas assigned to tickets; different stage types.

### Settled earlier (still standing unless reopened)

- Editing an approved stage does not withdraw its approval (the human, 2026-10-17 ~8:05 AM ET: "No").
- Default stages by kind ("Sure", 2026-10-16): withdrawn. No rule; stages are chosen per ticket
  at creation (the human, 2026-10-17).
- Who writes the proposal: the PM, most of the time; with an advisor live, or in the background.

### Open questions (advisor tj7q's list; recommendation in brackets)

1. Epic split and names: MVP = the schema on the ticket, files in a folder, the next-stage command
   for any agent, signed human gates, read in the existing docs viewer. V1 = the ticket page in
   bridle-ui (tabs, combined task/ticket page, approve button), template-heading check. V2 =
   attachments of any type (images), gates owned by agents, standard schemas to assign. [yes]
9. Existing tickets: the workflow applies to tickets created after it ships only. [yes]
10. Names: "stage" for a step, "file" for its output, "gate" for what must be true to pass,
    "schema" for the ticket's list of stages; RACI terms for who does what at a stage (the
    human, 4:20 PM ET: "we should use RACI terms for these as well. Helpful to be clear"). [see
    the chat proposal]

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
  write/approve and, when a stage becomes unblocked, messages the project manager once ("br-oooo:
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
br-oooo  Title                         state  priority  [actions: ready, hold, drop ...]
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
