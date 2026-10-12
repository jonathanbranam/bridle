+++
id = "br-g5dm"
title = "Products: a product is a set of projects with one roadmap and one product manager; several product managers"
kind = "feature"
state = "claimed"
created_at = "2026-10-09T20:06:43.304Z"
updated_at = "2026-10-12T09:44:57.368290Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wg5dm",
]
priority = "low"
priority_at = "2026-10-09T20:11:04.169227Z"
ticket = "g5dm"
+++

docs/tickets/open/products-a-product-is-a-set-of-projects-with-one-roadmap-and-g5dm.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T20:11:04.169Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-12T03:56:21.716Z
From the human, via advisor (product-manager), 2026-10-11 ~11:55 PM ET, on why the epics weren't moving: "You are the PdM, right? you are setting the sschedule. if these tickets can be worked, then work on them; if not, then lmk what I need to clarify." PdM decision under that go: run the designer on g5dm (epic `products`, roadmap #5), low priority, behind btdn/ajp5/ga6s; keep it flexible while the PdM trial runs; the design goes to the human for review before any build.

### note · external:orchestrator · 2026-10-12T03:56:41.396Z
From orchestrator: br-g5dm readied on the human's go (quote on the thread, via advisor/product-manager). Design pass only (designer role), low priority, behind btdn, ajp5 and ga6s; the design goes to the human for review before any build. Keep it flexible while the PdM trial runs (docs/notes/product-manager-trial.md).

### note · agent:pm-1 · 2026-10-12T03:56:52.877Z
Brief (pm-1): a design pass only, for the designer role; no code. Model: Sonnet. Low priority, behind btdn, ajp5 and ga6s.

Do: write the design into this ticket's body under a "## Design" section (the usual designer format: problem, what exists, options with the rejected ones named, a recommendation, questions for the human). Answer the "Open for the design" list: where a product is declared and how a project names it; where the roadmap lives and how the gateway shows it (bridle-ui ticket ui-vhrb); theme slugs per product; how the product manager role (br-6h65) is scoped to one product (tokens, daemons watched, inbox); work that spans products. Keep it flexible and small: the PdM trial is still running, so prefer the smallest declaration that works (a field in the home project's config.toml, say) and say what it defers (YAGNI). Honour the decided points above: a project can be product-less; requests for a project with a product go to the PdM, otherwise to the project aide.
Read first: docs/notes/product-manager-trial.md, docs/notes/roadmap.md (how themes and epics are used now), br-6h65, br-22ab (theme, parent), docs/design/agent-host/principals.md for roles.
Acceptance: the Design section is in the ticket; `bridle ticket check` clean for it; the done note lists the questions for the human. A migration note for any new config (xebc): automatic migration or "none, field is optional".
Out of scope: building anything, an enforced theme list (later, per the human), changing the PdM trial.
