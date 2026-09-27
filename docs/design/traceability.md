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
