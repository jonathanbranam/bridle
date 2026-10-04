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
