# Architecture: stable, revisable only with the human

> **Status (checked 2026-11-21):** Built and in use: the `arch-guard` hook, live for every spawned, resumed or renewed agent because the daemon passes the layer hooks at spawn (`Config::layer_hooks`); `bridle ticket land` refuses a branch touching `design/architecture/**` unless the task is an `arch-revision` · Built, not wired in: `bridle workflow arch list|propose`; the `arch-guard` hook (`bridle hook arch-guard`, in `workflow/base/hooks/PreToolUse.json`). No project has a `design/architecture/` tree yet · Planned: the human plan gate for `arch-revision` (no gates exist, see [[docs/design/gates|gates]])

The architecture tier holds what every task has to respect: principles, the
governing invariants (*the engine referees every rule*), component boundaries,
and major decisions recorded with the alternatives that were rejected. Each
element has an id and may be marked `invariant`.

It is **largely immutable, but open for revision**. It is not frozen, but it
never changes as a side effect of other work:

- Any edit under `design/architecture/` requires a task of kind
  `arch-revision`, and that task's plan gate is the human ([[docs/design/gates|gates]], planned). This is
  meant to be enforced, not only stated. The `bridle arch-guard` PreToolUse hook stops workers editing those files
  outside an `arch-revision` task (live for every spawned agent: the daemon passes layer hooks at spawn, no `sync` needed), and the integrator refuses to merge a branch
  that touches them unless its task is an `arch-revision` (human approval of the revision isn't checked: no gate records it).
- An agent that thinks the architecture is wrong runs `bridle workflow arch propose`,
  which creates an `arch-revision` task containing its argument. It then either
  continues within the current architecture or blocks its own task on the
  revision. It does not work around the architecture silently.
- An accepted revision starts the downstream re-evaluation described in [[docs/design/traceability|traceability]].

## Format and `bridle workflow arch list` (built)

An element is a level-2 heading with a hand-written id, and an optional
`invariant` flag, in any `*.md` under `design/architecture/`:

```markdown
## The engine referees every rule   {#a-12cd invariant}
Body text up to the next heading.

**Alternatives rejected:** a paragraph (up to the next blank line), kept as text.
```

`a-` ids are lowercase hex, four or more digits, written by hand and never
assigned by bridle. A heading without an id, a malformed id, an unknown flag,
or an id used twice (within a file or across files) is an error reported as
`file:line:col: message`. Parsing lives in `bridle-spec` (`arch`), sharing
`Diagnostic` and the id rules with the spec parser.

`bridle workflow arch list [--invariants] [--root DIR] [--json]` prints the elements
(local, no daemon); `bridle workflow arch propose --title T --argument TEXT|-` creates an
`arch-revision` task (daemon); see [[docs/design/cli|the CLI]]. The
`bridle arch-guard`, the
PreToolUse hook shipped in `workflow/base/hooks/PreToolUse.json` (matcher
`Edit|Write|MultiEdit`, rendered by `bridle workflow sync`), denies a worker's edit under
`design/architecture/` unless one of its claimed tasks is an `arch-revision`; the denial
tells it to run `bridle workflow arch propose`. Paths are resolved lexically against the hook's
`cwd`, so `../` tricks don't get around it. Non-worker principals are allowed, as is
everything on any error of bridle's own. The integrator check is built: `bridle land`
refuses a branch touching `design/architecture/**` unless the task is an `arch-revision`.
