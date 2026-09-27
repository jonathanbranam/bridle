# Architecture: stable, revisable only with the human

The architecture tier holds what every task has to respect: principles, the
governing invariants (*the engine referees every rule*), component boundaries,
and major decisions recorded with the alternatives that were rejected. Each
element has an id and may be marked `invariant`.

It is **largely immutable, but open for revision**. It is not frozen, but it
never changes as a side effect of other work:

- Any edit under `design/architecture/` requires a task of kind
  `arch-revision`, and that task's plan gate is the human ([[docs/design/gates|gates]]). This is
  enforced, not only stated. A PreToolUse hook stops workers editing those files
  outside an `arch-revision` task, and the integrator refuses to merge a branch
  that touches them without a linked, human-approved revision.
- An agent that thinks the architecture is wrong runs `bridle arch propose`,
  which creates an `arch-revision` task containing its argument. It then either
  continues within the current architecture or blocks its own task on the
  revision. It does not work around the architecture silently.
- An accepted revision starts the downstream re-evaluation described in [[docs/design/traceability|traceability]].
