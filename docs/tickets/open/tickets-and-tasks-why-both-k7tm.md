---
id: k7tm
title: Tickets and tasks, why both, and is that the long-term plan
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [ticket-state-without-moving-files-p2ys, which-docs-live-in-bridle-and-which-in-markdown-hv8e, refining-a-task-with-the-human-before-it-ships-hvxk]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> And all of this should be associated with the ticket or task. I'm still kind of unsure about
> why we have tasks and tickets separately, and if that's the right thing to do or if that's the
> long-term plan. Something to look into as well

## Today (advisor, 2026-09-29, at f5cd08b)

- A ticket is a markdown file in `docs/questions/` or `docs/spikes/` (verbatim source, notes,
  a Resolution naming where the answer landed). Since P0-6 (tskm), every ticket also has a
  bridle task whose body starts `original id: <id>`, and the task is the queue
  (`docs/README.md`). New work can be a task with no ticket.
- So each ticket is written down twice, and the two can drift.
- Related, still open: [[ticket-state-without-moving-files-p2ys|ticket state without moving
  files]] and [[which-docs-live-in-bridle-and-which-in-markdown-hv8e|which docs live in bridle
  and which in markdown]].

## To look into

Whether the long-term plan is one record per piece of work (the task, with its markdown
readable and editable in vim or Obsidian, per p2ys) or keeping both, and what each is for if
both stay.

## The human, again (2026-10-03, via the advisor)

After the advisor filed four raw ideas (8r5x, z485, 2tpm, 67qw) and each got a task straight
away, verbatim:

> Yeah, interesting. I don't know that they need tasks yet. Um, so I kind of want to understand
> that. Why you created tasks for them immediately. When I, I, like I was hoping to just create
> tickets only. Um, the task can stay open. I think it's, uh, it's fine. Uh, since they're
> already... created and uh, I yeah this the delineation between tickets and tasks is still
> actually if, if there isn't a ticket to talk about that then make make a make one of make a
> question uh, I think ticket um, for that that like do we really need both of these things and
> what lives on a ticket versus a task.

Why the tasks were made: the advisor role says to file each ticket "with its `bridle task new`",
`docs/README.md` says every ticket has a task (since tskm), and `bridle ticket new` creates the
task by default (`--no-task` skips it). Nothing distinguishes a raw idea, one that needs
refinement with the human before anyone could build it, from work ready to queue.

Added questions:

- Does a raw idea need a task at all, or only a ticket until it's refined and specified?
- What lives on a ticket and what on a task?

## Decided (2026-10-03, the human via the advisor)

The advisor laid out how it works today (a ticket's task is created `open`; it reaches the queue
only when planned and tiered; but the product-manager role "decides what is ready to build" and
"keeps the queue full", so it can plan any open ticket on its own judgement) and recommended:
`ticket new` makes no task by default; the task is created when a ticket is ready to work; the PM
plans only those. The human, verbatim:

> Uh, we need to do some changes, I think, pretty quickly on how tickets and tasks are work and
> are related. Um, creating a task for every ticket automatically is a bad behavior, I think. Um,
> so I'm considering disabling that immediately and then thinking about a better solution. But
> um, we can talk through it real quick. The problem is a bunch of tickets don't really need
> tasks, I don't think. Um, unless we're just using, unless we're using tasks to communicate or
> send messages about tickets. So that's the only reason I could think that a ticket would need a
> task. Um, there's another question on whether tickets and tasks need to remain distinct
> entities, but that's a bit of a different thing. Um, the other the other thing I'm only
> considering is that maybe certain types of tickets would automatically get a task. But in
> either case, whether we create the task immediately or not, it needs to be an intentional step
> to mark that task like ready for planning or whatever. Um, I know we added a five-minute wait on
> task work, and that's really good. Um, but you know, creating a ticket, even for a feature
> request and a task, uh, it shouldn't be, shouldn't fall into the schedule instantly. Um, it
> should be a choice that, hey, this is now ready to be worked.

> Yeah, that sounds good. I agree that with the change, don't create a task when a ticket is
> created. Um, as for who can create the task and mark it as ready for work, certainly the I
> mean, it probably depends a little bit, but the orchestrator or advisors definitely have that
> permission. And there's another task out there that has possibly been done, I'm not sure, but
> we're going to rename and change the roles and responsibilities of the current product manager
> into a project manager. And then I think we're going to pull that product manager role out and
> rework it so that it fulfills more of what a product manager should actually be doing. So
> there's a kind of a nuanced answer here, but I would say at some point a product manager can
> decide that the ticket's ready for a task, but currently that that shouldn't be the that that
> needs to come through me for now in one of my interactions with with an agent a ticket without
> a task should not be worked for now not without my approval but a task that's marked ready or
> whatever status is is the one we use that's when that's when the project manager can go
> schedule it

So, for now:

1. **`bridle ticket new` creates no task** (opt in with a flag). `docs/README.md`, the `tickets`
   rule and the advisor role stop saying every ticket has a task.
2. **A ticket without a task is not worked.** Creating its task and marking it ready is an
   intentional step, taken only with the human's approval, in one of their conversations with an
   agent: the orchestrator or an advisor does it.
3. **The PM (to become the project manager,
   [[the-product-manager-role-is-really-a-project-manager-who-hel-7r2c|7r2c]]) schedules only
   tasks marked ready.** It doesn't decide a ticket is ready itself. Later a reworked product
   manager may get that call; not now.
4. Kind-based automatic tasks: not now.

The human, clarifying (2026-10-03, verbatim):

> none of this should change some of our standing rules, which is that you know if there's a
> critical issue, the orchestrator can create a task and schedule the work to fix something.
> That's still within the orchestrator's set of permissions and allowed things to do. But for work
> that's you know like feature work or larger changes, they should always be run by me first.

5. **Unchanged: critical fixes.** For a critical issue the orchestrator still creates a task and
   schedules the fix itself. Feature work and larger changes always go to the human first.

Still open: which existing state means "ready" (`planned`, or a new one), what happens to open
tasks already made from tickets, and the rest of this ticket (whether tickets and tasks stay
separate).
