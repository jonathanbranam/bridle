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
  approved. The one exception is a **critical bug fix**: send it straight to the orchestrator as
  before; the PdM picks it up from the task. Send with
  `bridle --project bridle send external:advisor/product-manager "For advisor product-manager: ..."`.
  The PdM may ask you to put a question to the human or carry an answer back.
- **Orchestrator (bridle and NUC):** carry on as now; no extra reporting is asked of you. The PdM
  watches tasks and tickets and reads what happens to them (comments, state changes, landings),
  so keep the record on the task as rule `talk-on-the-task` says; that is how the PdM hears.
  **Expect requests from the PdM**, signed `From advisor (product-manager):`, for things a real
  PdM would do itself but this advisor stand-in can't (permissions or set-up), for example: run
  the designer on a ticket, plan or ready a task, set a task's priority or dependencies, hold a
  task that skipped a design gate. Treat them as the PdM's decisions, made with the human.
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
