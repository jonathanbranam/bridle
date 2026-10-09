# Product manager trial log

Status: working notes. A running log kept by advisor (product-manager), a stand-in for a
future product manager (PdM) role, so the real role can be designed from what the stand-in
actually did. Not a design; nothing here is built.

## Why this file exists

The human, 2026-10-09 (06:45 ET), verbatim:

> great, you are a fill-in for a future product manager role; one of your primary
> responsibilities is to document your experiences as a temporary PdM so that we
> can build the role effectively. Start a new file immediately in the docs/
> folder. We need a new folder here, docs/context has non-context files; I suggest
> docs/notes/ here is my explanation of what and why we need this role:
>
> I want to see a clear design for br-yfv5. Is the design role ready yet? I want a design
> written and attached to the task with a draft of the specs as well;
>
> I've suggested MIME types for adding multiple documents in a single task/ticket. Is there
> anything done on that work? Similar to an attachment, which should also be possible in the
> future - attaching images to tickets or tasks would be a future goal.
>
> I want to sequence this work properly - this is something for the product manager role -
> sequencing work that is NOT READY to ship; IDK how this lives exactly but we need to start on
> this. Work should be organized into a roadmap and tickets assigned to a theme or area of the
> roadmap, with sequencing on what is defined well enough to proceed to be worked on and which
> parts need my review first.
>
> in this case, e.g., the goal is to ship scheduled messages; but to ship that, in reverse
> order, we need:
>
> 1. a system scheduler
> 2. specs on the task, based on an approved design
> 3. human approval of the design
> 4. a design attached to the task
> 5. a way for documents to be attached to a task (and ticket); MIME approach
> 6. specs for the attachment appraoch, based on approved design
> 7. human review, amendments, and final approval of the design
> 8. a design written into the body of the task (since we don't have attachments yet at this
>    point)
> 9. a design role that writes designs
>
> this is a separate workstream than the performance issues and UI issues, etc. utlimately, i
> want a product manager role overseeing this with a way to store and define this sequence as a
> workflow in bridle (IDK what bridle workflows are yet, i got distracted when digging into that
> with other issues). This is an example of a software development workflow with HITL gates and
> multiple agents collaborating.
>
> For this task, as a trial, let's use existing solutions to do it; the
> advisor/product-manager will stand-in for a future PdM role and document the
> process and suggest how the role should work and fit into the bridle workflow.

## What the role is (as the human framed it)

- Owns work that is **not ready to ship**: sequencing, not dispatching. (The orchestrator
  dispatches ready work; the PdM gets work to "ready".)
- Keeps a **roadmap**: themes or areas, each ticket assigned to one.
- For each theme, knows which parts are **defined well enough to build** and which **need the
  human's review first** (HITL gates).
- Eventually stores the sequence as a **bridle workflow**, so the gates and hand-offs between
  agents (design role, human, workers, scheduler) are machine-tracked rather than in a head.

## Context for agents

> Rewritten as the trial goes; this section is current, the log below is history. Last
> rewritten 2026-10-09 07:35 ET.

**The experiment.** From 2026-10-09 the human is trialling a product manager (PdM) role. Until a
real role exists, a Claude Code session stands in for it: **advisor (product-manager)**,
address `external:advisor/product-manager`, on the bridle project's daemon. It uses only
existing tools (tickets, tasks, comments, `bridle task dep`, ticket `needs:`) and records what
works here so the real role can be designed from it. The trial starts strong (all work through
the PdM) and will back off as the human sees what works.

**What the PdM owns.** All work's path to the roadmap: a roadmap of themes, each ticket placed
in one, and for each theme the order of steps, which steps are defined enough to build and
which wait on the human's review (design, specs, approval). The first theme is scheduled
messages (yfv5): designer role, design in the ticket, the human's review, specs, then build.
The roadmap will be at `docs/notes/roadmap.md`.

**How to work with the PdM:**

- **Aides (every project):** send **everything** from the human to the PdM, not the
  orchestrator: ideas, feature asks, changes, "should we build X", and work the human has
  approved. The one exception is a **critical bug fix**: send it straight to the machine's orchestrator (see below) as
  before; the PdM picks it up from the task. Send with
  `bridle --project bridle send external:advisor/product-manager "For advisor product-manager: ..."`.
  The PdM may ask you to put a question to the human or carry an answer back.
- **Orchestrator:** one per machine, not per project (the human, 2026-10-09: "There is only one
  orchestrator per machine; we need to make that clear. the orch for this machine runs in the
  bridle project."). Dalek's runs in the bridle project, so from another project send to it with
  `bridle --project bridle send external:orchestrator ...`; the NUC has its own. There is no
  "bridle-ui orchestrator". The orchestrator: carry on as now; no extra reporting is asked of you. The PdM
  watches tasks and tickets and reads what happens to them (comments, state changes, landings),
  so keep the record on the task as rule `talk-on-the-task` says; that is how the PdM hears.
  **Expect requests from the PdM**, signed `From advisor (product-manager):`, for things a real
  PdM would do itself but this advisor stand-in can't (permissions or set-up), for example: run
  the designer on a ticket, plan or ready a task, set a task's priority or dependencies, hold a
  task that skipped a design gate. Treat them as the PdM's decisions, made with the human.
- **Any agent asking for a feature or bug fix in bridle or bridle-ui** (the trial's product):
  send it to the PdM, not the orchestrator or aide (the human, 2026-10-09: "when other agents ask
  for a feature or bug-fix for a project with a product, those requests go to the PdM; otherwise
  they go to the project aide"). Critical fixes still go straight to the orchestrator.
- **Everyone:** keep the conversation in the system: comments on the task or ticket, state
  changes, `task ask`. Direct messages are for important context that doesn't fit there. The PdM
  takes the human's next steps to them directly or through the project's aide. Nothing from the
  PdM overrides the human's own words; if the two disagree, ask.

**Keep this across restarts.** Put this line in every handover note until it's withdrawn:
"PdM trial on: read 'Context for agents' in bridle's docs/notes/product-manager-trial.md."

## Log

### 2026-10-09 06:45 ET: started

- Created this file and `docs/notes/` (the human's suggestion; `docs/context/` has drifted
  into non-context files).
- First workstream: **scheduled messages** (br-yfv5). Next: find what exists for each of the
  nine steps above, using only existing tools (tickets, tasks, `needs`, comments, messages).

### 2026-10-09 07:00 ET: where the scheduled-messages ladder stands

Checked each of the human's nine steps against tasks, tickets and git (main at b1c37432):

| Step | Status |
|---|---|
| 9. a design role | **Done.** `designer` (br-ukpm) landed 2026-10-09 03:13 ET, commit 5fa2d459, `workflow/base/roles/designer.md`. Never run yet; its planned first job is fne2. It writes options and a recommendation **into the ticket**, not a design doc (the human's words in ukpm: "I don't want a design document. I want a ticket."). |
| 8. design in the body | Matches the designer's output, but in the **ticket** body, not the task's. Tickets hold design, tasks hold work (rule `tickets`, k7tm). |
| 7. human review of the attachment design | Not started: nothing to review. |
| 6. specs for attachments | Not started. Spec tooling (`bridle spec`, `design/specs/`) is built but not wired into any role. |
| 5. attachments, MIME approach | **No ticket.** The only MIME mention is in x8jt (document review): comments threaded inline in a doc, "like how MIME types are attached to an email". Nothing on several documents or images per ticket or task. |
| 4.-2. design, approval, specs for yfv5 | Not done. |
| 1. a system scheduler | **Partly built, out of order.** br-9xze (per-project scheduled messages, `bridle schedule add/list/rm`) landed 2026-10-09 ~03:05 ET with decisions written by the orchestrator into the task body and approved by the human 2026-10-08. br-g5y2 (role priming to use it) is pending. The rest of yfv5 (cbbn nightly restarts, 3nyk maintenance windows, cy2v machine-wide) has no design. |

What exists today for sequencing (no new code):

- Tickets: `needs:` (ordering between tickets) and `see:`. No theme or roadmap field.
- Tasks: `bridle task dep` (coordination edges), `pending` until the human (or PM) runs
  `task ready` (an HITL gate), `task ask` (a blocking question).
- `docs/proposal/build-order.md`: the only roadmap-like doc; phase-based, and ma2x says it marked
  phases built that nothing uses.
- Workflows: `workflow/base/` holds roles and rules in layers (`docs/design/workflow-layers.md`).
  Layer `workflow.toml` parses only `layer = ...`: **no gates, stages or sequences can be stored
  as a workflow today.**
- Naming: the existing `product-manager` role is really a project manager (7r2c, decided to
  rename to project-manager). This stand-in is the product manager the human meant there.

Lesson 1 for the role: **work slipped past the gates because the gates weren't written down.**
The orchestrator moved the first scheduler slice to build with the human's approval of the
slice, which was right by today's process, but the design-then-specs ladder the human now wants
had no artefact to check against. A PdM's first job is to write the ladder down before the
workforce reaches it.

### 2026-10-09 07:35 ET: how agents learn about the trial

- Decided with the human: a message to each orchestrator and aide pointing at "Context for
  agents", plus a line in every handover note. No temporary role edits (they'd reach every
  project through the base workflow and need a sync to add and to remove).
- The human tightened the first draft, verbatim: "I really want all work (possibly excepting
  critical fixes) to go through the PdM. I think critical fixes do also - they impact the
  roadmap and delivery; they could be directly scheduled, but the PdM should be notified (the
  PdM should be an auto-watcher on most tickets; we'll have to define "most", maybe start with
  watching all tickets for now)." And: "instead of demanding communication from the orch ...
  PdM should watch all tickets; messages can be sent for additional, important context, but
  let's keep communication in the system; comments on tasks; status changes on tasks; etc should
  "communicate" not lots of DMs". And: "Let's start strong then back off later ... You should do
  as much as you can without orhestrator; but you might lack permissions in an advisor role; orch
  should expect requests from you that would normally be directly done by the PdM but that you
  lack permissions or set up for."
- Lesson 2 for the role: **the PdM's input is the record, not messages.** It should be an
  automatic watcher of every task (to be narrowed later), so a real role needs auto-watch at
  task creation; the stand-in has to `bridle task watch` by hand.

### 2026-10-09 07:45 ET: trial announced

- Pushed this doc (790a19ac). Sent the pointer to external:orchestrator (m-7666),
  external:orchestrator@nuc (m-7667) and the aides of bridle (m-7668), track-web, bridle-ui and
  meta-notes (outbox o-0034..36).
- Watching all 103 open tasks (`bridle task watch`, by hand, one by one). Gaps for the real
  role: the watch is recorded as `external:advisor`, an identity every advisor shares, not as
  the PdM; and tasks created from now on won't have the PdM as a watcher unless someone adds it.
  A real role needs its own principal and auto-watch at task creation.
- The bridle aide acknowledged (m-7669) and handed over what it had open with the human for the
  PdM: max_workers 2 -> 3 (aide recommends no until the v6kr baseline), whether to start br-yfv5
  then br-g5y2, npj2 follow-ups, q7mv recommendations, closing br-a3b9, and this morning's new
  tickets rjd5, 22n2, 9xbk, 57nt.
- Watching works: task comments reach the PdM as messages that end its wait (first: m-7673,
  br-r9h7 done by jxaffix). With 103 tasks watched this will be noisy; the real role needs
  either a digest or a filter (state changes and human-gate events only), to be judged once
  there's a day of volume.

### 2026-10-09 07:55 ET: first item routed through the PdM

- The bridle aide sent br-g3az (docs fix: status line token setup writes an empty file), with
  the human's go verbatim ("yes fix the docs."). No design gate needed (a doc correction), so the
  PdM watched it, readied it (`pending` -> `open`) and put the approval on the thread. The
  advisor stand-in could do `task ready` itself; no orchestrator request needed.
- Lesson 3: most items need no ladder. The PdM's test is "does this need a design or the
  human's review before build?"; if not, pass it straight through and get out of the way.
- v6kr (system architect role, resource baseline): the aide added the human's memory-measure
  question and the macOS-correct measures to the ticket and asked the PdM to make sure the
  baseline uses them. Done as a comment on br-v6kr (planned), so whoever picks it up reads it.
  The PdM's job here is carrying a human's concern from the ticket to the work, on the record.

### 2026-10-09 08:30 ET: bridle-ui bugs, and the planning gap

- ui-kqsp (Document page renders blockquotes and lists wrong; the human hit it reading this
  file) and ui-5zrr (comment highlights break markdown). Both are bugs the human asked to have
  fixed, with the approach in the tickets, so no design gate. Readied both, set priority high
  (first among bridle-ui work, the aide's ranking), noted on each to do them together, and
  asked the orchestrator to plan them (m-7732): the advisor can ready and prioritise but not
  plan (`task plan` is the project manager's).
- Lesson 4: bridle-ui has no project manager agent of its own (only manager-2), and the aide's
  two earlier messages to the orchestrator on 5zrr went unanswered. A PdM that can see every
  project's queue catches work that falls between projects; a real role needs cross-project
  task visibility (`--project` on every call works today but is by hand).

### 2026-10-09 09:25 ET: planning permission, a provisional role, and the roadmap

- The human asked whether to "breakfix" so the PdM can plan tasks, or make a provisional PdM
  role. Found: the daemon doesn't stop an advisor from `task plan` (only `task ready` is
  role-checked, and advisors may ready). The limit is the role split, not permissions: planning
  (size, model, files, brief) is the project manager's craft (pm-1 plans every `open` task
  automatically), and the PdM's lever is `ready` plus priority and order. The real gap was that
  bridle-ui has no project manager, so its readied tasks had no one to plan them; the orchestrator
  acts as PM there. (Both bugs landed within the hour anyway.)
- Advice given: no breakfix and no provisional daemon role yet. The PdM is mostly conversation
  with the human, which an interactive session does well and a background role does badly. What
  the stand-in lacks is its own identity (watches are recorded as the shared external:advisor)
  and auto-watch. Revisit when 6h65 (the product manager role ticket) is designed.
- Roadmap: `docs/notes/roadmap.md`, 12 workstreams, every open task in bridle, bridle-ui and
  track-web placed in one. Tables generated from live task state by a scratch script, then kept
  by hand. Lesson 5: the roadmap wants to be a view over task data (a workstream field on the
  task or ticket, plus a "next gate" field), not a document; the doc is the prototype of that
  view.
- Found two existing tickets that are this trial's subject: 6h65 (a product manager that
  relates every ticket to open and planned work; the human: "I think that should be product
  manager, new role, not project") and 95mu (a change spec reviewed before any worker builds:
  the general form of the scheduler ladder).

### 2026-10-09 ~10:15 ET: a priority workstream, machine setup

The human, verbatim: "Do you have a work stream for project setup, like inter-machine project
setup, token setup, automatic syncing? If not, please create one. And I want to get that work
delivered. Put tickets in there that would enable basically efficient and pretty direct setup
for a new machine. And I want to get that work scheduled and address any issues on it. So build
a roadmap, add that as a work stream to the roadmap. And I want that as a priority so that I
can use it to set up the new Windows machine and add it to the network. So it would include
everything for configuring background daemons, launching tokens between machines, tokens
between projects, and then as a as a additional feature I would like to see if we can get it
in is automated project transfer between machines."

- The roadmap had these tasks spread under "Many projects, many machines". About 25 open
  tickets touch machine setup. A subagent is surveying them (state, dependencies, today's
  end-to-end procedure, gaps) before the PdM proposes the workstream's order.
- Lesson 6: the first real PdM job is not inventing work but **collecting it**: the tickets
  for a goal already exist, scattered, filed one complaint at a time. The value is the
  sequence and the gaps.
- Survey back (a Claude Code subagent). Most building blocks exist (systemd units, WSL guide,
  doctor checks, Tailscale re-check); the glue is missing: no token mesh command, no single
  procedure, no project move. Filed hua2 (add-a-machine guide), xrkh (systemd uninstall; an
  owner refusal must not crash-loop the old machine's unit), kt25 (project move, needs design).
  Readied what was buildable and set priority high; asked the orchestrator to schedule.
- Lesson 7: "make it a priority" turned into four PdM actions with today's tools: ready,
  priority, a comment carrying the human's words on each task, and one message to the
  orchestrator. Nothing in bridle records that a set of tasks is one goal; the roadmap doc is
  the only place.

**2026-10-09 ~1:20-1:40 PM ET.**

- Restart (h-0078), then mostly watching: f4xu, 6nzj, h7gt delivered; pm-1 planned all of
  machine-setup phase 1. Roadmap updated on each.
- The human asked "how are things progressing?". Checking found hua2 done at 12:35 but not
  merged, manager idle, and the free slot given to a normal bug (b6mu) over the high phase-1
  tasks. Asked the orchestrator; the answer: main was red on another flake (vabu), so nothing
  could merge; order now vabu, hua2, then phase 1.
- Lesson 8: watching task events is not enough to see a **stall**. No event fires when a done
  task sits unmerged or a priority is passed over; I saw both only because the human asked.
  A real PdM needs a "time in state" check on its priority workstream (e.g. done-but-not-merged
  over 30 minutes), not just a feed of changes.
- The bridle-ui aide asked why ui-ha6m and ui-wtr3 (the human's asks, filed ~10:30 PM last
  night) were still pending. Nobody readies another project's pending tasks: the aide files
  them, the orchestrator waits for ready. Readied both; answered ui-9hq8 (code deps landed;
  waits on the human's NUC token, which br-8c25 automates).
- Lesson 9: **pending is a dead end without an owner.** Every project's pending tasks need
  someone sweeping them (the PdM, here), or the human's asks sit unnoticed overnight.

**2026-10-09 ~3:00-3:15 PM ET.**

- Restart (h-0082). Advisor (tickets) wrote the human's answers to 22ab Q1-Q5 into the ticket;
  updated the plan table to match (step 5, stx8, out of the workstream; purge deferred). The plan
  itself still waits on the human's approval before steps 1-4 are filed.
- br-8z7j (one pusher) was HELD for build, but landing the designer's docs-only branch marked the
  task integrated. Asked the orchestrator to reopen it held; done.
- Lesson 10: **a design step and a build step share one task, so landing the design closes the
  work.** The task's state can't say "design landed, build owed". Either the designer works on
  the ticket without a task branch of its own task, or design and build are separate tasks (or
  tickets: 22ab's child tickets would make this natural).

**2026-10-09 ~3:30 PM ET.**

- The human asked why the comment counter (br-gd43) was high. It was my overrating: "live bug
  since delete landed", but the human can't delete comments yet. The human: "all of the comment
  work is low priority; should come at the end of queue for other work." Set low: br-gd43,
  ui-vnuu, br-pa8h.
- The human, on why: "the issue with less important working going ahead is that the \"easy\" and
  \"small\" changes still run builds and peg the CPU so it is quick for the worker to implement,
  but it actually adds 30-40min." Passed to the orchestrator: order by priority, never by size.
- Lesson 11: **size is not cost.** A task's cost to the human's queue is its build and check time
  on the shared machine (~30-40 min, whatever the diff), not its coding time. A real PdM ranks by
  value and treats every task as a fixed machine cost; "it's small, slot it in" is the wrong
  instinct. Check before raising priority that the bug can actually hit the human today.

**2026-10-09 ~5:20 PM ET.**

- The human settled the terms (d9wq): the roadmap orders **epics** (outcome, "Done when"),
  grouped by **themes** (lasting areas, slugs). Roadmap restructured; 22ab plan approved and steps
  1-4 filed (syqn, bpku, 3v75, 72t9); new epic `migrations`.
- Asked whether themes and epics cross projects, the human named a new thing: a **product**, "a
  collection of projects (1:1 with github repos) that share a single roadmap and product
  manager". Epic g5dm. The reason is context, the agent's and the human's: a PdM for bridle
  shouldn't carry track-web's work.
- Lesson 12: **a PdM's scope is a product, not a project or the machine.** This trial watches
  every project on the machine (bridle, bridle-ui, track-web, dotfiles-local), which is already
  more context than one roadmap needs.
