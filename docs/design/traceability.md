# Traceability: from goals down to tests

Each tier links upward to the one above it:

```
goal  g-03
  └─ architecture element  a-12cd        serves: g-03
       └─ requirement  r-7fa2             traces: a-12cd@3f9e
            └─ scenario  s-b310           (belongs to r-7fa2)
                 └─ test                  bound by scenario id (see specs-to-tests)
```

Links are written inline in the lower element, next to its id:

```markdown
### Requirement: The engine referees every rule    {#r-7fa2 traces=a-12cd@3f9e}
```

The `@3f9e` is a short hash of the upstream element's text **as of when this
link was last confirmed**. That is what makes change tracing mechanical (the
*suspect link* idea, borrowed from requirements-management tools):

- When an upstream element's text changes on merge, every link pointing at it
  with an old hash becomes **suspect**.
- `bridle trace suspect` lists suspect links. When an `arch-revision` is
  accepted, bridle opens one `re-evaluate` task per affected capability, listing
  its suspect requirements.
- For each requirement, the worker on that task either **confirms** it
  (`bridle trace confirm r-7fa2`, which rewrites the hash and records that it is
  still valid) or **edits** it, which may in turn make its scenarios suspect.
- An `arch-revision` task's impact ([[docs/design/impact-and-conflicts#The impact registry|impact registry]]) is the whole downstream set, so the
  conflict check warns every in-flight task on an affected spec before the
  revision merges, not afterwards.

Queries:

```
bridle trace down a-12cd     # everything that depends on this element
bridle trace up s-b310       # why this scenario exists, up to goals
bridle trace orphans         # requirements tracing to nothing (warning, configurable)
bridle trace coverage g-03   # how much of this goal has design, specs, tests
```

## Built

- **Link syntax**: `traces=<id>@<hash>[,...]` on requirements, `serves=<id>[,...]` on
  architecture elements. Unknown link targets and malformed links are errors.
- **Text hash** (`bridle_spec::trace::text_hash`): the first 4 hex digits of the SHA-256 of the
  heading title (no `{#id ...}` block), a newline, and the body up to the next heading, with
  whitespace runs collapsed to one space. Goals, architecture elements (alternatives included) and
  requirements (prose before the first scenario) all hash this way.
- **Queries**: `bridle trace down|up|orphans|suspect|confirm` (local, see `docs/design/cli.md`).
  Not built yet: `coverage`.
- **Re-evaluate tasks**: when a task of kind `arch-revision` is marked done, the daemon
  (`bridle-daemon/src/reevaluate.rs`, called from `done_task`) computes the suspect links over
  `design/{goals,architecture,specs}` in the repo checkout (the landed commit, since done requires
  it on the integration branch), groups the suspect requirements by spec file, and opens one
  open (not planned) `re-evaluate` task per file: `re-evaluate <file stem> after <arch task id>`,
  the requirement ids and the confirm-or-edit instructions in the body. No suspects, no tasks; a
  task with that title already existing is not duplicated. The manager (the human if none runs)
  gets a note naming the new tasks. Trace inputs that fail to parse or link are logged and skipped.
