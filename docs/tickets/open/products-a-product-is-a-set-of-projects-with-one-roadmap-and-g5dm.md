---
id: br-g5dm
title: "Products: a product is a set of projects with one roadmap and one product manager; several product managers"
kind: feature
created: 2026-10-09
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-6h65, br-d9wq, br-22ab, br-syqn]
next_comment_id: c8
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

## Decided: product-less projects

The human, 2026-10-09 ~5:45 PM ET, verbatim: "a project can be product-less; a project w/o a
product does not get those features: roadmap, product-manager role, themes; it has tickets and
maybe epics; it can have a project-manager and other features of bridle."

So the product is optional. Without one, a project has no roadmap, no product manager and no
themes; it keeps tickets (perhaps epics), the project manager and the rest of bridle. Example:
dotfiles-local.

## Decided: where requests go

The human, 2026-10-09 ~5:50 PM ET, verbatim: "when other agents ask for a feature or bug-fix for
a project with a product, those requests go to the PdM; otherwise they go to the project aide."

## Open for the design (not decided)

- Where a product is declared (its home project's config? a machine-level registry?) and how a
  project names its product.
- Where the roadmap lives in the home project, and how the gateway shows it (see bridle-ui
  ticket ui-vhrb: theme page and roadmap page).
- Whether theme slugs are scoped per product (likely) and checked against a list.
- How the product manager role (br-6h65) is scoped to one product: tokens, which daemons it
  watches, its inbox.
- Work that spans products: one owner, others notified?

Priority (the human, same message): "the right direction BUT NOT URGENT; let's keep working with
what we have - it's working well for use currently." Low.

Needs a design (the designer) and the human's review before any build. Learning from the PdM
trial (docs/notes/product-manager-trial.md) feeds it.

An enforced list of themes comes later (the human, 2026-10-09 ~5:40 PM ET: "agree no enforced
list of themes yet, but that should come later"); it likely belongs with this epic, since themes
are per product.

## Design

Status: design for the human's review; nothing is built. Designer pass (worker wg5dm), 2026-10-12.
Kept small on purpose: the PdM trial (`docs/notes/product-manager-trial.md`) is still running,
and every choice below is a field or a convention that can be replaced without a migration.

### Problem

Today one PdM keeps one roadmap (`docs/notes/roadmap.md`, by hand) for everything bridle runs.
The human wants several products (bridle + bridle-ui; meta-notes + meta-notes-ui + notes;
track-web alone), each with its own roadmap and PdM, so neither the agent's context nor the
human's mixes unrelated work. Some projects have no product (dotfiles-local), and some work
crosses products.

### What exists

- A project is one repo with one daemon and a registry entry (`~/.bridle`); the machine config
  has a `[projects]` table for cross-machine addresses (principals.md, "Addressing").
- `theme:` and `parent` are ticket fields (br-22ab, `bridle ticket new --theme`, `ticket set`).
  A theme is a free slug, not checked against any list. Epics and the roadmap are kept by hand
  in `docs/notes/roadmap.md`.
- Roles are scoped per project: a role's token and inbox belong to one daemon. External
  principals (advisors) hold one key per project (principals.md).
- Requests between projects go to a named agent (`bridle send`) or to the project's aide.
- The PdM role does not exist yet (br-6h65); advisor/product-manager is the stand-in.

### Decisions already made (the human, in the ticket)

A product is optional. A product-less project has tickets, maybe epics, a project manager and
the rest of bridle, but no roadmap, PdM role or themes. A request for a project that has a
product goes to the PdM; otherwise to the project aide.

### 1. Where a product is declared, and how a project names it

Options:

- A. A machine-level registry of products (`~/.bridle/products.toml`) listing member projects.
  Rejected: it is per machine, so each machine needs its own copy and they drift; products
  are a property of the work, not of a laptop.
- B. A `[product]` table in every member project's `.bridle/config.toml`, each naming the
  product. Needs no home project, but the members can disagree and nothing says where the
  roadmap is.
- C. **Recommended.** The product is declared once, in its **home project's**
  `.bridle/config.toml`; every other member names it with one line.

> [!comment] c1 human, 2026-10-16 22:12 EDT, on "Recommended. The product is declared once, in its home project's .bridle/config.toml; every other member names it with one line." [read 2026-10-16 22:27 EDT]
> Agree. Approved C. Product declared in a home project (user decides).

>
> **doc-g5dm, 2026-10-16 22:31 EDT:** @human Noted, C stands. Nothing to change in the text.
>
> **resolved by doc-g5dm, 2026-10-16 22:31 EDT**
```toml
# home project (the one holding the roadmap), e.g. bridle
[product]
name = "bridle"             # slug; unique among the human's products
home = true                 # this project holds the roadmap

# member project, e.g. bridle-ui
[product]
name = "bridle"
home = "bridle"             # the home project's name, so a member can find it
```

Simplified: `name` is required; `home` is `true` in the home project and the home's project
name in a member. No `[product]` table means product-less. It travels with the repo (git is the
source of truth) and is read by every machine that clones it. A member naming a product
whose home it cannot reach (not on the registry) is a warning in `bridle doctor`, not an error.
Membership is therefore the set of projects whose config names the product; the daemon never
needs a list. Projects are 1:1 with repos, so a repo cannot be in two products (see 5).

Migration (xebc): none. The table is optional and absent means product-less, so existing
projects need nothing.

### 2. Where the roadmap lives, and how the gateway shows it

The roadmap lives in the home project. It stays where it is for bridle
(`docs/notes/roadmap.md`) today and moves to a fixed `docs/roadmap.md` in the home project
(the human, 2026-10-16); the `[product]` table has no roadmap path. It stays a
hand-kept document owned by the PdM for now; tasks in member projects are listed in it by
qualified id (`ui-vhrb`, `br-g5dm`), which the id scheme already makes unambiguous across
projects. A database-backed roadmap is deferred (YAGNI): the trial has not settled its shape.

Gateway (bridle-ui ticket ui-vhrb, theme page and roadmap page): the roadmap page is chosen
by product, read from the home project's daemon (document read already crosses projects in
`bridle-docs`). The gateway lists products by scanning the configs of the projects it already
knows; a product-less project shows no roadmap or theme pages. Nothing new is needed on the
wire beyond `product` (name, home, roadmap path) on the project's info response; that is the
only API change, and it is additive.

### 3. Theme slugs per product

Themes are scoped to the product: `product-process` in bridle and `product-process` in
meta-notes are different themes that happen to share a slug. A ticket's `theme:` is
interpreted in its project's product; a product-less project's `theme:` is ignored by the
roadmap and a warning from `bridle ticket check` (the human decided themes belong to
products). The qualified name used outside a product's context is `<product>/<slug>`
(`bridle/product-process`). No enforced list yet (the human, 2026-10-09: later). When it comes,
the natural place is a `themes = [...]` array in the home project's `[product]` table, checked
by `bridle ticket check`; deferred, not rejected.

### 4. Scoping the product manager role (br-6h65)

The PdM is a seat belonging to the **home project's daemon**, not a separate kind of daemon:

- **Tokens.** One external-principal key per member project (the same mechanism advisors use).
  Each key is a daemon API token: the PdM reads and files tickets through each member daemon's
  API (`bridle` CLI against that daemon), not by reading the member's files, so it reaches each
  member daemon and no others. Product-less
  projects and other products' projects have no key for it. Created when a project joins the
  product (`bridle token create --external product-manager/<product>` on the member).

> [!comment] c4 human, 2026-10-16 22:17 EDT, on "the PdM reads and files tickets in each member daemon and in no others" [read 2026-10-16 22:27 EDT]
> Is the token for the daemon? How does the pdm read files? The file system or bridle?

>
> **doc-g5dm, 2026-10-16 22:31 EDT:** @human Yes, the token is for a daemon: one per member project. The PdM goes through bridle (each member daemon's API), not the file system. I reworded the Tokens bullet to say so.
>
> **resolved by doc-g5dm, 2026-10-16 22:31 EDT**
- **Daemons watched.** Exactly the member projects found by section 1, plus none else. The PdM
  waits like every other role, through the unified per-role waiter, not a PdM-specific poll or
  watcher (the human, 2026-10-16).

> [!comment] c2 human, 2026-10-16 22:14 EDT, on "Daemons watched. Exactly the member projects found by section 1, plus none else. Watching uses the existing task watcher mechanism (br-519b), not a new poll." [read 2026-10-16 22:27 EDT]
> I don't think this matters any more. There should be a unified waiter for each role no matter what.

>
> **doc-g5dm, 2026-10-16 22:31 EDT:** @human Agreed. I replaced the br-519b watcher sentence with the unified per-role waiter, in Daemons watched.
>
> **resolved by doc-g5dm, 2026-10-16 22:31 EDT**
- **Inbox.** One inbox on the home daemon. The PdM runs on the home project, so its address is
  its ordinary principal there, `external:product-manager` (the human, 2026-10-16), not a new
  address form. Requests for a member project go there, from any sender, instead of the project
  aide (the decided routing):
  `bridle send` and the request-a-feature path resolve the recipient from the target project's
  `[product]` table, falling back to the aide when it is absent. The member's aide keeps its
  own inbox for project-internal matters, so nothing is lost if the PdM is not running (and
  per br-6h65, the PdM need not be standing: a message to its inbox starts it).

> [!comment] c3 human, 2026-10-16 22:16 EDT, on "One inbox on the home daemon, address product-manager/<product>@<home>. Requests for a member project go there instead of the project aide (the decided routing)" [read 2026-10-16 22:27 EDT]
> Since the pdm runs on the project home it already has a name external:product-manager of the bridle project.

>
> **doc-g5dm, 2026-10-16 22:31 EDT:** @human Right. I dropped the `product-manager/<product>@<home>` address; the inbox is the PdM's ordinary principal on the home daemon, `external:product-manager`.
>
> **resolved by doc-g5dm, 2026-10-16 22:31 EDT**
- Rejected: one PdM with a "current product" switch. That keeps every product in one context,
  the problem this epic exists to remove.

### 5. Work that spans products

One owner, others notified. A task belongs to exactly one project, hence to one product; the
owning PdM sequences it. If it touches another product, its ticket lists that product's
project in `repos:` (already a field) and the owner's PdM sends a one-line note to the other
PdM's inbox naming the ticket; the other PdM records a dependency or a "watch" in its roadmap
and decides nothing about the work. No shared epic across products: if the work is large,
each product gets its own epic and they are linked with `related`. Rejected: a cross-product
roadmap, which recreates the mixed context.

### What this defers (and why)

- Enforced theme list: the human said later; section 3 names where it goes.
- Roadmap in the database and auto-generated pages: the trial's hand format is not settled.
- A `bridle product` CLI (list, add member): `[product]` in config plus doctor is enough until
  there are two products to manage.
- PdM auto start and stop: owned by br-6h65 / w2hj, not this design.
- A project in two products: not allowed; the 1:1 repo to project rule makes it unnecessary.

### How to verify a build of this

A project with and one without `[product]`: `bridle doctor` accepts both; the info response
carries `product` only for the first; a message addressed "request" to a member project lands
in the PdM inbox, to a product-less project in the aide's; the gateway shows a roadmap page
only for the product.

### Questions for the human

1. Is `[product]` in `.bridle/config.toml` (home declares, members name it) the right place, or
   do you want a machine-level registry despite the drift problem?
2. Should the roadmap file move to a fixed `docs/roadmap.md`, or keep the configurable path
   (bridle's stays `docs/notes/roadmap.md`)?

> [!comment] c5 human, 2026-10-16 22:17 EDT, on "Should the roadmap file move to a fixed docs/roadmap.md" [read 2026-10-16 22:27 EDT]
> Yes docs roadmap

>
> **doc-g5dm, 2026-10-16 22:31 EDT:** @human Done. Section 2 now says a fixed `docs/roadmap.md` in the home project, with no path option. bridle's file moves there from docs/notes.
>
> **resolved by doc-g5dm, 2026-10-16 22:31 EDT**
3. Are themes per product with the qualified form `<product>/<slug>` right, and should
   `bridle ticket check` warn on a `theme:` in a product-less project?
4. Cross-product work: is "one owner, the other PdM notified, no shared epic" enough, or do you
   want a real shared epic?

> [!comment] c6 human, 2026-10-16 22:18 EDT, on "Cross-product work: is "one owner, the other PdM notified, no shared epic" enough" [read 2026-10-16 22:27 EDT]
> Yes.

>
> **doc-g5dm, 2026-10-16 22:31 EDT:** @human Good, the section 5 text stands.
>
> **resolved by doc-g5dm, 2026-10-16 22:31 EDT**
5. Should a message from one agent to a member project reach the PdM even when the sender is in
   the same product (a bridle-ui worker asking bridle), or only for outside senders?

> [!comment] c7 human, 2026-10-16 22:19 EDT, on "Should a message from one agent to a member project reach the PdM even when the sender is in the same product (a bridle-ui worker asking bridle), or only for outside senders?" [read 2026-10-16 22:27 EDT]
> Don't understand. Agents should message the product-manager agent on the home project of the product. Anyone can message anyone

>
> **doc-g5dm, 2026-10-16 22:31 EDT:** @human My question was whether a sender in the same product is treated differently. Your answer is no: requests for a member project go to the PdM on the home project, from any sender, and no one is blocked from messaging anyone. I added "from any sender" to the Inbox bullet.
>
> **resolved by doc-g5dm, 2026-10-16 22:31 EDT**