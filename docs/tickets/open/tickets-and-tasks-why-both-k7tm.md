---
id: k7tm
title: Tickets and tasks, why both, and is that the long-term plan
kind: question
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [ticket-state-without-moving-files-p2ys, which-docs-live-in-bridle-and-which-in-markdown-hv8e, refining-a-task-with-the-human-before-it-ships-hvxk]
tasks: [br-3724, br-9e15]
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

The human, settling what "ready" means (2026-10-03, verbatim):

> I thought that, I don't know, explain how it works today. I think what you said is that an open
> task can be scheduled by the PM and the PM is who decides when to plan it. I think that's true.
> Um, in some cases, the orchestrator can, you know, plan a urgent task. But generally, tasks
> should be marked as ready to work. I think that means open. And then either the PM should plan
> them. And then that's when the manager gets involved. Unless we're fast tracking a bug fix or of
> some kind. Does that answer the question? I, I think that's correct. So a ticket doesn't get any
> work on it because there's no task. But as soon, yeah, I'm a little fuzzy on the difference
> between a task and a ticket still. So let's stick with this for now. A task, if we've created a
> task, that means it's ready for work.

6. **A task's existence means ready for work.** No new state: an `open` task is ready; the PM
   plans it (`open` -> `planned`) and tiers it; then the manager picks it up. The orchestrator
   can plan an urgent fix itself. A ticket with no task gets no work.

Still open: the 112 tasks open on 2026-10-03, most made automatically from tickets before this
decision (including br-2b0b, br-ab3a, br-fd74, br-4eaf, raw ideas marked "do not build"). Under
rule 6 they'd all count as ready; they need sorting (keep the ones the human approved, drop or
park the rest). And the rest of this ticket: whether tickets and tasks stay separate.

Sorting them (the human, 2026-10-03, verbatim, choosing "pm-1 lists them" from the advisor's
options):

> Sure, let's go with number two, but for any tasks that the PM decides shouldn't be worked on,
> just immediately drop the tasks. Don't confirm with me. Only confirm with me for the tasks that
> the PM thinks do need to be worked on.

Relayed to pm-1 the same day: drop at once what it judges shouldn't be worked (the ticket stays);
send the human one list of the rest; plan only what the human approves.

## A kind for "design before build"? (2026-10-03)

Filing [[scheduled-messages-an-agent-or-the-human-schedules-a-message-hrcn|hrcn]], the human
asked for a kind meaning "a design for a new feature", "because that's something we need to have
a lot of", then: "Maybe that's just called a feature ticket, but there's no task." Under the rule
above, a `feature` ticket with no task already means "not ready to build". Open: add a `design`
kind anyway, or let "no task" carry it.

## The first task takes the ticket's ID (the human, 2026-10-03)

After learning that task IDs (`br-` + 4 hex) and ticket IDs (4 from
`abcdefghjkmnpqrstuvwxyz23456789`) use different alphabets (see
[[ids-the-human-can-say-aloud-task-and-ticket-ids-that-survive-nkd9|the speakable-IDs ticket]]),
verbatim:

> Okay, that's too bad that task IDs and ticket IDs have a different alphabet. That's definitely
> something to follow up on, so let's make a ticket about that. There's already a ticket about
> the difference between tasks and tickets, so add that information there.
>
> Another comment or suggestion from me is that when we're creating a task from a ticket, we
> should just use the ticket ID for the task. That's not going to always work, but I think in the
> majority of cases, we have one task for most tickets. I guess I would say we do. I'd have called
> out that not every ticket will have exactly one task, and that's still definitely true.
>
> I think it would be useful to have a mapping, too. Now that we still have a map between tickets
> and tasks that's explicit, we should keep that. I think we should have a rule that the first
> task that gets created from a ticket uses the ticket's ID. That'll make things a lot easier for
> me.
>
> That has a couple of implications, obviously:
>
> * Aligning the alphabets.
> * When creating a ticket, it should check for tasks that collide as well, if that's
>   straightforward enough.
>
> I don't know. Is the ticket ID a superset of the task ID? I presume it is. What are the letters
> that are left out of task IDs behind me? It's probably L and O or something.

Proposed (not built; needs design and the human's go-ahead):

1. **The first task made from a ticket uses the ticket's ID** (`br-k7tm` for ticket `k7tm`).
   Later tasks from the same ticket get fresh IDs. The explicit map stays both ways: the ticket's
   `tasks:` frontmatter and the task body's `original id:`.
2. **One alphabet for both.** Today they differ:
   - Ticket IDs: 31 characters, `a-z` and `2-9` without `i`, `l`, `o`, `0`, `1`.
   - Task IDs: 16 characters, hex `0-9a-f`.
   - **Not a superset:** every hex letter (`a-f`) and the digits `2-9` are in the ticket alphabet,
     but task IDs also use `0` and `1`, which ticket IDs leave out. So about 41% of today's task
     IDs couldn't be ticket IDs. Moving new task IDs to the ticket alphabet fixes that;
     existing task IDs keep theirs.
   - The speakable-IDs ticket may change the alphabet again; settle the two together.
3. **One ID space for collisions.** `bridle ticket new` checks the new ID against existing task
   IDs too (it already talks to the daemon to file a task, so a lookup is cheap), and the daemon
   refuses a new task whose ID is a ticket's unless it is that ticket's first task.


## The task races the ticket body (2026-10-03)

Reported by the NUC orchestrator for the human (m-3977). On meta-notes, `bridle ticket new`
(without `--no-task`) minted the stub (empty "The ask") and its `open` task at once; the manager
picked the task up within seconds, twice (mn-caa0, mn-bfc6: tickets bmen, gjf6), before the body
was written and pushed, and asked why the ticket was empty. The human, verbatim: "if we have the
ability to create a task immediately from the ticket, that task goes immediately to open, and
that's going to cause the manager or somebody to pick it up. I don't think that makes a lot of
sense. There's something wrong there in how we've designed this solution."

Cause: decision 1 above (`ticket new` creates no task) isn't built yet; `crates/bridle/src/ticket.rs`
still files the task unless `--no-task`. Under decision 6 an `open` task means ready, so a task
born with the stub is ready before the ticket says anything. The 5-minute settle applies only to
`planned` tasks, so it doesn't help here.

Proposed (one small task, needs the human's go):

1. **Build decision 1.** `ticket new` never files a task; drop `--no-task` (accept it as a no-op
   for a while so scripts and role text don't break).
2. **Creating the task is its own step, after the commit:** `bridle ticket task <id>` files the
   task from the ticket (title, kind, link both ways, as `new` does today). It refuses when the
   ticket's "The ask" is empty or the ticket file isn't committed on the integration branch, so a
   worker can never find an empty or unpushed ticket. This is the intentional "now it's ready"
   step decisions 2 and 6 describe.
3. **`ticket new --body`/`--body-file`** (optional, cheap): write the ask at creation so a stub is
   never empty to begin with.
4. Docs: `docs/README.md`, the `tickets` rule, cli.md, the advisor and orchestrator roles (who may
   run `ticket task`: the orchestrator or an advisor with the human's approval, or for a critical
   fix).

Rejected: a draft/held task state. Decision 6 says a task's existence means ready; a held state
brings back the ambiguity the human wants gone.

## Approved: one alphabet, one ID space (the human, 2026-10-03)

> Okay, so I'm comfortable with the change to have tickets and tasks use the same alphabet and use
> the same IDs. That's fine. That, that work, I don't think, should be blocking anything else. Um,
> so it can go ahead.

Task **br-9e15** (open): new task IDs use the ticket alphabet; a ticket's first task takes its ID;
`ticket new` and the daemon share one collision space.

## The question to explore next: merge tasks into tickets?

The human, verbatim (2026-10-03), to take up in a fresh session:

> The other question is essentially there are tickets that don't need to be worked and that's
> something that will stay around forever. There may be some tasks occasionally that don't have
> tickets associated with them. I don't, I think that's okay, but I guess that would be a question
> is how often does that occur? Is that something we still need to support? And then the next
> question is like, what's the dividing line between a ticket and a task? We're only scheduling
> tasks, but what else does a task do? You know, I feel like a task is a work item and it makes
> sense that a work item has a different workflow. But is that distinction like something we need
> to keep? Like, what could we just, you know, take the functionality of tasks and add it to
> tickets and then get rid of tasks entirely. That's the question I want to explore.

**The main question: could tickets take over what tasks do, and tasks go away?** Sub-questions:

1. **Tasks without tickets: how often, and do we still need them?** Count on 2026-10-03 (475 tasks,
   rough, by the task body and title):
   - 103 linked formally (body starts `original id: <ticket>`);
   - about 241 more name a ticket loosely (a `(xxxq)` in the title, or `Ticket: docs/tickets/...`
     in the body);
   - about **131 (28%) have no ticket at all**, 101 of them created by agents (follow-ups, fixes,
     test flakes, worker-found bugs) and 30 by external principals (the orchestrator, advisors).
2. **What does a task do besides being scheduled?** Inventory for the exploration: states
   (open, planned, claimed, integrated, dropped, reopened), settle period, claim and release,
   queue tiers, edges (`blocks`, deps), questions and answers that block it, a thread
   (notes, comments), watchers and wakes, priority and size, the human's to-dos (`--for-human`),
   impact and conflicts, `land`, incidents (a task kind), submissions from other projects
   (visitors), the summary, events. Which of these need a database record, and which could live
   in a ticket's markdown?
3. **The dividing line.** A ticket is a lasting record (the human's words, the reasoning,
   decisions) that may never be worked; a task is a work item with its own workflow. Is that
   difference worth two records, or one record with an optional work workflow?
4. **What merging would cost:** tickets are markdown files in git (readable in vim and Obsidian,
   reviewed in commits); tasks are daemon state (fast queries, wakes, concurrency, the state
   branch). Related: [[ticket-state-without-moving-files-p2ys|p2ys]],
   [[which-docs-live-in-bridle-and-which-in-markdown-hv8e|hv8e]],
   [[everything-readable-and-editable-through-the-daemons-file-ba-v8kn|v8kn]],
   [[tickets-and-docs-in-bridle-s-repo-or-a-separate-repo-or-subm-tkav|tkav]].

Already decided above, and unaffected by the answer: a ticket without a task gets no work; a
task existing means ready; the human approves feature work.


## Approved: `ticket task` as its own step; no `design` kind (the human, 2026-10-03)

On "The task races the ticket body" and "A kind for design before build", verbatim:

> Okay, I agree in point number one, that can be built as a small fix. to point number two. A
> ticket that requests a new feature is implicitly not ready to be built until, in the current
> system, until the task is created. We might be changing that, but I agree.

So: the four-part proposal under "The task races the ticket body" is approved as one small fix
(sent to the orchestrator). No `design` kind: a `feature` ticket with no task already means "not
ready to build".
