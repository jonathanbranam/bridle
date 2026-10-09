---
id: syqn
title: Ticket fields get standard names (blocked_by, related, parent, created, resolved), a theme field and project-qualified IDs
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [22ab, zkbb]
tasks: [br-syqn]
---

## The ask

Parent: br-22ab (step 1 of its plan; the human approved the plan 2026-10-09 ~4:40 PM ET). The design is in 22ab; this ticket is the change. Read 22ab sections 7 (links: the field renames), 8 (IDs) and 11 (migration, step 1), and its "Plan" section's note on themes.

Scope:

- Frontmatter field renames, decided by the human: `needs` -> `blocked_by`, `see` -> `related`, new `parent` (one ID), `opened` -> `created`, `filed_by` -> `created_by`, `closed` -> `resolved`; `kind`, `repos` stay. `changes` and `specs` are dropped by later steps, not this one; `tasks` stays until step 2.
- New field `theme: <slug>` (the human, 2026-10-09): a theme is a lasting area named by a short, readable slug (`reliability`, `human-ui`, `agents-and-cli`), not an ID. For now the list of slugs lives in `docs/notes/roadmap.md`; `ticket check` warns on an unknown slug only if that's cheap, otherwise accepts any slug.
- Project-qualified IDs (section 8): `id: br-k7tm` in frontmatter, link values, threads, messages and the UI; bare `k7tm` accepted as input where unambiguous and expanded; refused with candidates where ambiguous. Prefixes unique across registered projects (refused at registration). File names stay bare (`<slug>-<id>.md`); `ticket check` matches the file name's tail to the `id:` without its prefix.
- `bridle ticket new/set/check` write the new names and read both old and new until the migration has run everywhere.
- A migration (xebc, `crates/bridle/src/migrate.rs`) renames the fields and qualifies the IDs in every project's tickets. Other projects' commits go to their trial or integration branch, never `main` (rule `existing-projects`).
- Docs in step: `docs/README.md` (ticket conventions), `docs/design/cli.md`, rule `tickets`.

Out of scope: ticket types and readiness (step 2), links indexed from frontmatter (step 3), the thread move (step 4).

Acceptance: `just check`; `bridle ticket check` clean on bridle after the migration; an old-format ticket still reads.

## Open question: themes and epics across projects

The human, 2026-10-09 ~5:05 PM ET, verbatim: "do themes cross projeccts??? do epics? damn.
overlooked that one".

They already do in practice: the roadmap's epics have tasks in bridle and bridle-ui
(everything-is-a-ticket's step 8 is a bridle-ui ticket), and theme `multi-machine` holds ui-9hq8.

PdM recommendation, for the human to confirm before this step builds the theme field:

- **Themes are the human's, not a project's.** A slug means the same theme in every project
  (`human-ui` in bridle and bridle-ui is one theme). KISS: no central registry to start; each
  ticket names a slug, and the gateway (which already spans projects) groups by it. `ticket
  check` accepts any well-formed slug.
- **An epic lives in one project and its children may live in any**: `parent: br-22ab` on a
  bridle-ui ticket. Project-qualified IDs (section 8 of 22ab) make that work with no new design.
- **The roadmap order is cross-project too**, so it can't live in one project's repo. Until a
  home is designed (the gateway, or a machine-level file), it stays in bridle's
  `docs/notes/roadmap.md`.
