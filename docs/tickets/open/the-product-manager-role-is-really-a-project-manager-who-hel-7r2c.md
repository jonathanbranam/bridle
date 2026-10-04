---
id: 7r2c
title: The product-manager role is really a project manager; who helps the human decide what to build
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [sk52]
tasks: [br-8b6c]
---

## The ask


The human, verbatim (2026-10-02, via advisor workflow), in the discussion of
[[when-instructions-reach-an-agent-at-start-or-at-each-step-op-sk52|sk52]]:

> I feel like there should be, we may need to, to kind of rephrase the difference between a
> product manager and a project manager. I think I, I used the wrong term there. Um, I think what
> the current PM role does is more of a project manager managing the roadmap and delivery. I think
> instead we need a product manager who would work with me on what's appropriate for this
> project. Um, what's the appropriate thing to build and how to build it? Um, or we can just use
> advisors for that with the appropriate prompts. That, that might just be easier.

**Not sent to the orchestrator.** Discussion only.

## Today

- `workflow/base/roles/product-manager.md`: owns the backlog and the queue, triages tickets,
  prepares tasks, reviews new tickets (including xebc's migration-plan check). That's roadmap and
  delivery: a project manager's job, as the human says.
- The split came from
  [[docs/tickets/resolved/split-the-manager-into-product-and-development-managers-tx3f|tx3f]]
  (resolved; its task `br-db9a` is still open, which looks stale).
- Working with the human on what to build and how is done today by the advisor (this session)
  or the orchestrator, with no product-specific instructions.

## Options (advisor, not decided)

1. Rename `product-manager` to `project-manager` (config, role file, the PM-or-human queue gate
   `require_pm_or_human`, docs), and add a product role later if needed.
2. Keep the role, and give advisors the product work through step skills (sk52): an advisor
   running the ticket or design step is the product partner, with no new role. The human's "might
   just be easier".
3. Both: rename now, product work through advisors and skills.

Advisor's lean: 3 once sk52's skills exist; the rename alone is cheap but touches an existing
project's config (meta-notes, track-web), so it needs a migration (xebc).

## Decided (2026-10-02)

The human, verbatim: "Yeah let's rename it to project manager can be shortened to proj-mgr if
needed in naming."

Rename `product-manager` to `project-manager` (`proj-mgr` where a short name is needed). Bridle's
own repo, and the existing projects through a migration (xebc; they're the human's existing
projects, so the migration is reviewed). Sent to the orchestrator. The product-partner work
(option 2) waits on sk52. Renaming or splitting the advisor is handled elsewhere, by another
session.

## Next steps (advisor workflow, retiring, 2026-10-04)

The rename is the human's decision (2026-10-02, "Yeah let's rename it to project manager"), sent to the orchestrator as m-3788, but no implementation task exists: pm-1 dropped this ticket's question task br-8b6c on 2026-10-03 in the k7tm sort ("not approved for work now"). The orchestrator was asked again to file and queue the rename task, with the migration for existing projects reviewed by the human first. The product-partner half waits on sk52.

## The product manager's job, per the human (2026-10-04)

The human, verbatim: "I need all of these things listed out somewhere for follow up; what's the
best place? I think a todo (human claimed task for me); in the future this should be managed by a
product manager - their role and resp. is to keep an eye on open tickets; tasks that need follow
up; and who is responsible so I can ask them: what's the latest with feature xyz or ticket kj3d
and get an answer."

So the product role (distinct from the renamed project manager, which runs the queue and
delivery) would own: the open tickets, which need follow-up, who is responsible for each, and
answering the human's "what's the latest with <feature or ticket>?". Interactive, per the human's
earlier "an interactive product manager possibly who can talk to me about the plan" (2026-10-03).
The case that prompted it: the follow-ups from 34bw were tracked by hand in the human to-do
`br-tkph`, because their question tasks were dropped in the k7tm sort. Not decided; not sent.

## A job for the future product manager (the human, 2026-10-04, via advisor (tickets))

Said while resolving [[tickets-and-tasks-why-both-k7tm|k7tm]], verbatim:

> yes resolve it; if there is any further follow up, create a todo for me (in the future - this
> would go to the product manager to handle pending ticket and status updates and follow up on work
> that needs human decision or has other dependencies)

So the reworked product manager would own: pending tickets and tasks (what waits for the human's
`task ready`), status updates, and following up on work that needs the human's decision or waits on
other dependencies, filing the human's to-dos for those. Until that role exists, advisors file the
to-dos.


## A worked case: x8jt, run by an advisor (the human, 2026-10-04, via advisor doc-review)

The human, verbatim, retiring the advisor that ran x8jt:

> in the future - a peice of related work like this should be managed by the product manager;
> find the ticket for that work and update it with some details about what we've done here and
> how a product manager would manage the pending and future work.

**What was done by hand.** x8jt (document review) went from idea to built in about a day, run
by one advisor (doc-review) with the human:

- The advisor took the human's decisions in conversation (comment format, batching, tags, the
  sent mark, review now) and recorded each on x8jt verbatim, then sent approved slices to the
  orchestrator to file, since an advisor can't mark tasks ready or file in bridle-ui.
- The orchestrator filed seven tasks across two repos (br-rp53, br-aj9d, br-5paw, br-pwtw,
  br-qttb; ui-acf0, ui-c39e); pm-1 planned them; the advisor relayed progress from the
  orchestrator's messages.
- At the end nothing tracked what was left: the setup on the human's machine and the first trial.
  The advisor checked each task's state by hand and filed the human's to-do br-twg8 with the
  next steps and a prompt for the next advisor, because sessions were restarting and the
  advisor's context would be lost.

**How a product manager would run it.** One owner per piece of work (an effort: a ticket or a
few related ones), outliving any one session:

- **Owns the effort's record**: the ticket(s), every task filed for it in every repo, and who is
  on each. Answers "what's the latest with x8jt?" from that record, not from a session's memory.
- **Takes the human's decisions** (or receives them from the advisor the human talked to),
  records them on the ticket, slices approved work into tasks and has them filed in the right
  project and opened, without the advisor-to-orchestrator relay.
- **Watches the pending work**: tasks still `pending` for the human's `task ready`, tasks
  dropped in a sort, deferred items (here: thread IDs, showing what changed, diagrams yyzm),
  and dependencies (here v8kn and r9vh, which the slices sidestepped).
- **Notices when built isn't the same as in use**, and files the human's to-do for the steps
  only they can take (here: install the UI, start the gateway, run the first review), then
  follows up on it.
- **Hands over through the ticket**, so a restart costs nothing: what's built, what's next,
  who's on it. This is what br-twg8 did by hand. Seats (gtzx) would give the role a lasting
  identity and inbox across restarts.
