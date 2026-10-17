---
id: br-tj7q
title: "Stages MVP: tickets prepare themselves (version 2 tickets, stage files, signed gates, bridle writes each stage)"
kind: epic
created: 2026-10-17
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ej8y, br-ckbt, br-95mu]
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

## Scope: this epic is the MVP (the human, 2026-10-17)

Grilled with advisor tj7q, 2026-10-17 ~11:30 AM - 5:00 PM ET. The human split the work into this
MVP ("Tickets prepare themselves"), V1 br-ej8y ("Stages you can see and steer") and V2 br-ckbt
("Gates beyond the human"): "Both the splits and the overall description look good ... It would
be great to get the MVP done today so I can use it tomorrow."

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

## Outcome

A version 2 ticket prepares itself: bridle starts a designer for each stage as it unblocks, each
given that stage's instructions and template at the moment of writing; it stops only at the gates
the ticket lists for the human, who signs (traced) or sends a stage back; when every stage is
written and signed the ticket is ready by itself; a worker builds it from tasks.md and its specs
land in the project's bridle specs. Version 1 tickets keep today's workflow untouched.

## Done when

- A version 1 and a version 2 ticket can both be created, and each goes through its own workflow.
- One real version 2 ticket has gone from filed to landed through the stages, with the human
  signing at least one gate (br-ewk8).
- Version 2 is the default for new tickets (br-u2r8; the human: "I would add to the 'when done'
  for that epic that version 2 becomes the default").

## Children, in order

| Step | Ticket | What | After |
|---|---|---|---|
| 1 | br-9mqh | version, `stages:` schema, proposal section, stage folder, check, stages.md | - |
| 2 | br-quvh | four stage rules and templates; `bridle ticket next` | 1 |
| 3 | br-ussh | stage-done, approve (signed, traced), changes, restage; ready means prepared; watchers | 1 |
| 4 | br-ewhy | bridle spawns a designer per unblocked stage; 60-minute timer to the PdM | 2, 3 |
| 5 | br-6c8x | a worker builds a version 2 ticket from tasks.md (OpenSpec apply) | 2 |
| 6 | br-ewk8 | one real version 2 ticket end to end, with the human | 4, 5 |
| 7 | br-u2r8 | version 2 becomes the default (on the human's go after 6) | 6 |

2 and 3 can run side by side. Children are version 1 tickets (the human: "If we break ticketing
on version 2, we won't be able to work any tickets").

## Decisions (settled with the human, 2026-10-17; quotes verbatim, full words on the thread)

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
- **Ticket `version`** (the human, 4:28 PM ET: "Maybe we should add a version tag to a ticket ...
  That way, we can say new tickets must be version 2, but then the old tickets don't have to
  migrate"; 4:32 PM: "I prefer version it's very common"): frontmatter `version: 2`, absent = 1.
  `ticket new` writes 2; `ticket check` requires stages (at least the proposal section) on 2 and
  accepts 1 as it is. Old tickets never migrate; opting one in = setting version 2 and its stages.
- **Names** (settled 4:38 PM ET): stage (a step), file (its output), gate (what must hold before
  the stages after it start), schema (the ticket's list of stages). RACI words explain who does
  what in docs and tickets (writer = responsible, signer = accountable), but are not schema field
  names ("let's not use RACI terms in the schema. That is too extra"); the schema keeps `review:`.
- **Watchers are informed on every stage change** (the human, 4:36 PM ET, verbatim): written,
  waiting for a signature, signed, sent back, timed out.
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

Later (named by the human, not MVP; scoped in V1 br-ej8y and V2 br-ckbt): if the merged specs differ from the reviewed specs.md and
the specs stage had a review gate, the new specs go back to that reviewer, human or agent ("TBD if
that is only an inform or an approval"; 4:20 PM ET, "far future concern but important to follow
up"); the apply step gets its own layered rules injected at the moment of work, copying
OpenSpec's apply skill freely ("having rules injected right then is really helpful for certain
very tight programming things where we want the code written in a specific way. That should be
an expectation"); gates owned by agents (other kinds of review); standard
schemas assigned to tickets; different stage types.

### Settled earlier

- Editing an approved stage does not withdraw its approval (the human, 2026-10-17 ~8:05 AM ET: "No").
- Default stages by kind ("Sure", 2026-10-16): withdrawn. No rule; stages are chosen per ticket
  at creation (the human, 2026-10-17).
- Who writes the proposal: "most of the time, the proposal should be written by the product
  manager" (2:00 PM ET), narrowed for MVP at 3:25 PM: the filer may claim it and write it at once,
  otherwise bridle's designer writes it. The PdM writing it is a filer's claim.

## Supersedes or absorbs

- The scheduler ladder (roadmap, 2026-10-09), steps 2-8; the rest of the scheduler (yfv5) becomes
  a ticket run through this.
- Epic `attachments`: other files in the stage folder are attachments (MVP: markdown); any type,
  images first, is V2.
- br-95mu (proposal and design approved before planning): the gates the human lists, here. The
  v2va review slices stay separate.
- ui-fwsx (one ticket page) moved to V1, br-ej8y.

## History

The designer's '## Design options' (two passes, 2026-10-16/17, last in ce38c6ca) and the
advisor's first reading are superseded by the decisions above and removed from this body; read
them in git history. Where the decisions differ: the daemon spawns the writer (not "notify the
PM"); instructions and template are given at the moment of writing (not in the role prompt);
no default stages by kind; every ticket has a proposal, in its body.
