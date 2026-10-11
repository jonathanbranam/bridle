---
id: br-ydyk
title: ticket new in another project can write the wrong id prefix (br- in bridle-ui)
kind: bug
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-11T22:41:13Z
---

## The ask

Two bridle-ui tickets minted by external:aide on 2026-10-11 have `id: br-tecf` and `id: br-utp5` in
their frontmatter, while their rows are `ui-tecf` and `ui-utp5` (bridle-ui's `[tasks] prefix = "ui"`).
The mismatch breaks them: `bridle --project bridle-ui ticket show ui-utp5` fails with "ui-utp5 belongs
to project ui, not br; run this in ui's checkout", even run from bridle-ui's checkout, so the ticket
can't be planned through the ticket commands.

Likely cause (not confirmed): `own_prefix` in `crates/bridle/src/ticket.rs` takes the prefix from
whichever daemon `client_for(cli)` reaches, while the row is created on the project's own daemon. If
the two differ (identity or project resolved from a different source), the file gets the other
daemon's prefix.

Fix: the file's prefix and the row's must come from the same daemon, or `ticket new` must refuse when
they disagree. Acceptance: a test where `ticket new` runs against a project whose prefix differs from
the cwd's project writes the target project's prefix.

Found by external:orchestrator while planning ui-utp5.
