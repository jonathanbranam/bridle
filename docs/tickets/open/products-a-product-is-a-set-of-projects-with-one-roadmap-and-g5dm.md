---
id: g5dm
title: "Products: a product is a set of projects with one roadmap and one product manager; several product managers"
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [6h65, d9wq, 22ab, syqn]
tasks: [br-g5dm]
---

## The ask

The human, 2026-10-09 ~5:20 PM ET, verbatim (to advisor product-manager):

> thanks ok, so I do think we'll have some alignment of projects to specific roadmaps and themes. Some will cross projects, some will not; and I need a product-manager for different projects that is focused only on them and not distracted with other projects' work. However, to your point, some work spans multiple projects, necessarily. bridle and bridle-ui; some work might hit dotfiles-local. Some bridle work might affect multiple projects.
>
> I think the main thing is managing context (both the agent's and mine) - I will have a roadmap for track-web that has nothing to do with bridle; also i'll have a roadmap for meta-notes that includes probably meta-notes, meta-notes-ui and notes. Let's keep experimenting today, and keep the design flexible. For today, one PdM (you) and one roadmap, but we need an epic whose "done when" is: there will be multiple product managers, each responsible for a product that contains many projects; a product roadmap must "live" in a single place (probably a project), but its planning will span multiple projects.
>
> So, I accidentally invented a new thing, but I think it fits: a product: a collection of projects (1:1 with github repos) that share a single roadmap and product manager.

## The epic

- **Outcome:** each product has its own product manager and roadmap, so neither the agent's
  context nor the human's mixes unrelated products.
- **Done when** (the human): there are multiple product managers, each responsible for a product
  that contains many projects; a product roadmap "lives" in a single place (probably a project),
  but its planning spans multiple projects.
- **Theme:** `product-process`.

## Terms (the human, 2026-10-09)

- **Product:** a collection of projects (1:1 with GitHub repos) that share a single roadmap and
  product manager. Examples: bridle (bridle, bridle-ui, perhaps dotfiles-local); meta-notes
  (meta-notes, meta-notes-ui, notes); track-web on its own.
- Themes and epics belong to a product. Some work crosses products (bridle work that affects
  every project; a change in dotfiles-local); how that is handled is part of the design.
- Today: one PdM (the advisor product-manager trial) and one roadmap (bridle's
  `docs/notes/roadmap.md`). Keep the design flexible while the trial runs.

## Open for the design (not decided)

- Where a product is declared (its home project's config? a machine-level registry?) and how a
  project names its product.
- Where the roadmap lives in the home project, and how the gateway shows it (see bridle-ui
  ticket ui-vhrb: theme page and roadmap page).
- Whether theme slugs are scoped per product (likely) and checked against a list.
- How the product manager role (br-6h65) is scoped to one product: tokens, which daemons it
  watches, its inbox.
- Work that spans products: one owner, others notified?
- A project with no product (the human, 2026-10-09 ~5:35 PM ET: "probably something like
  dotfiles-local could live alone or even be product-less? IDK if that should be a thing").

Priority (the human, same message): "the right direction BUT NOT URGENT; let's keep working with
what we have - it's working well for use currently." Low.

Needs a design (the designer) and the human's review before any build. Learning from the PdM
trial (docs/notes/product-manager-trial.md) feeds it.

An enforced list of themes comes later (the human, 2026-10-09 ~5:40 PM ET: "agree no enforced
list of themes yet, but that should come later"); it likely belongs with this epic, since themes
are per product.
