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
