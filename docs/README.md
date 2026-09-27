# Bridle docs

> **Status: proposal.** The decisions in [[docs/proposal/decisions|decisions]]
> are the designer's. Everything else is a recommendation to be argued with.
> Open questions are one file each under `questions/open/`.
>
> **Split in progress.** Everything outside `agent-host.md`, `research/` and
> `spikes/01-*` was split out of `design.md` at **`c192bfc`**. `design.md`
> itself is left untouched until the v1 build finishes; see
> [Cutover](#cutover).

## Layout

```
docs/
  README.md          this file
  proposal/          why bridle exists, what is decided, what it's for, the build order
  context/           facts the design works within: the projects, the host, prior research
  design/            the high-level design, one topic per file
  agent-host.md      the v1 daemon + agent host. Source of truth for v1; not yet split
  research/          bridle's own research reports
  spikes/            finished spike briefs and findings (NN-*.md)
    open/            spikes not yet run — one ticket each
  questions/
    open/            open design questions — one ticket each
    resolved/        answered, with a Resolution section saying where the answer lives
```

## How the folders work

**Design and proposal docs are named by topic** and change in place, like
specs. They have no IDs and no frontmatter. Link to them by path:
`[[docs/design/gates|gates]]`, or `[gates](docs/design/gates.md)` inside a
table, where a wiki alias's `|` would break the row.

**Questions and spikes are tickets**, following
`workflow-instructions/ticket-conventions.md`:

- One file each, `<descriptive-tail>-<id>.md`, with a 4-character ID that
  never changes. Create them with `workflow-instructions/scripts/new-ticket.sh`
  (point `TICKETS_ROOT` at a folder with the right layout), or by hand with a
  fresh ID unique across the repo.
- The same frontmatter: `id`, `title`, `opened`, `repos`, `changes`, `specs`,
  `needs`, `see`. `needs:` orders them.
- The checker takes one ticket root, so `needs:` and `see:` only name tickets
  in the same tree (`questions/` or `spikes/open/`). A dependency across the
  two goes in the body as a `**Needs**` line with the ID and a link. That
  keeps it greppable.
- **The folder is the state.** Resolving a question is a `git mv` from `open/`
  to `resolved/`, plus a `## Resolution` section naming the design doc the
  answer landed in. The design doc, not the ticket, is the durable record.
  Before resolving, ask: is the answer written into `design/` or
  `agent-host.md`? A spike moves to `spikes/done/` once its findings doc
  exists beside the numbered spike docs.
- Link to tickets by full stem, never a bare ID:
  `[[where-the-single-orchestrator-lives-hj4g|where the orchestrator lives]]`.
- Bodies quote their source verbatim (the doc section, the human's words) and
  don't triage.

Check links and IDs with:

```bash
for t in docs/questions docs/spikes/open; do
  python3 ../workflow-instructions/scripts/check-tickets.py --root . --tickets $t --quiet
done
```

## Reading order

1. [[docs/proposal/problem|The problem]], [[docs/proposal/decisions|decisions]],
   [[docs/proposal/goals-and-non-goals|goals and non-goals]].
2. [[docs/context/projects|The projects]] and [[docs/context/nuc-host|the host]].
3. [[docs/design/overview|Architecture overview]], then the design files below
   as needed.
4. [[docs/agent-host|agent-host.md]] for what v1 actually builds.
5. `questions/open/` and `spikes/open/` for what isn't settled.

## Where each part of design.md went

The baseline is `design.md` at **`c192bfc`**. Headings lost their section
numbers, and `§` references became links. Nothing else in the text changed,
except for the notes added in the split, each marked as such.

| design.md @ c192bfc | Now |
|---|---|
| header, status | this README |
| §1.1 The problem | [problem](docs/proposal/problem.md) |
| §1.2 Decided | [decisions](docs/proposal/decisions.md) |
| §1.3 Goals, §1.4 Non-goals | [goals and non-goals](docs/proposal/goals-and-non-goals.md) |
| §2 The projects it has to serve | [projects](docs/context/projects.md) |
| §3 Architecture in one picture | [overview](docs/design/overview.md) *(+ split note)* |
| §4 The workflow as layered, modifiable data | [workflow layers](docs/design/workflow-layers.md) |
| §5.1 Roles, §5.2 Task lifecycle, §5.4 The loop | [roles and lifecycle](docs/design/roles-and-lifecycle.md) *(+ split note)* |
| §5.3 Gates | [gates](docs/design/gates.md) |
| §6.1–6.4 Edges, messages, questions, hearing | [coordination](docs/design/coordination.md) |
| §6.5 Worktrees and ports | [worktrees and ports](docs/design/worktrees-and-ports.md) |
| §7.1 The tiers | [knowledge tiers](docs/design/knowledge-tiers.md) |
| §7.2 Goals | [goals tier](docs/design/goals-tier.md) |
| §7.3 Architecture | [architecture tier](docs/design/architecture-tier.md) |
| §7.4 Traceability | [traceability](docs/design/traceability.md) |
| §7.5 Explorations | [explorations](docs/design/explorations.md) |
| §8.1, §8.2, §8.5 Specs, migration | [specs](docs/design/specs.md) |
| §8.3 Impact registry, §8.4 Conflict protocol | [impact and conflicts](docs/design/impact-and-conflicts.md) |
| §9 Specs to tests | [specs to tests](docs/design/specs-to-tests.md) |
| §10 Storage | [storage](docs/design/storage.md) *(+ split note)* |
| §11 Usage limits and token efficiency | [usage and budget](docs/design/usage-and-budget.md) |
| §12 The CLI surface | [cli](docs/design/cli.md) |
| §13 Skills | [skills](docs/design/skills.md) |
| §14 Build order | [build order](docs/proposal/build-order.md) |
| §15 Open questions 1, 3–13 | `questions/open/`, one ticket each |
| §15 item 2, §6.4 "to verify" | `questions/resolved/workers-as-subagents-or-separate-sessions-qb97.md` |
| §4.4 "verify the mechanism", §10.1 "questions/… open" | spike `vxp6`, question `c5a8` |

Also filed, without changing their source docs:

- `agent-host.md` §13: items 1, 3, 4, 5 as questions, item 2 as spike `akjw`.
- `research/01-agent-runtime.md` §8: spikes 2–7. Spike 1 is `spikes/01-*`.
- `spikes/01-stream-json-findings.md`, Risks and follow-ups: the spike-shaped
  items. Version pinning is build work, not a spike, and isn't filed.
- The 2026-09-27 review of the [[docs/context/nuc-host|NUC host]] against
  `agent-host.md`: four questions and two spikes.

## Cutover

Once the v1 build is done:

1. `git diff c192bfc -- docs/design.md docs/agent-host.md docs/research docs/spikes`
   shows what changed during the build. Carry each change into the file the
   table above names, and file new open items as tickets.
2. Replace `design.md` with a short stub holding the table above, so old
   `design.md §N` references, including those in `agent-host.md`, still lead
   somewhere.
3. Split `agent-host.md` the same way, probably into `design/agent-host/` by
   section, and move its §13 into the question tickets already filed here.
4. Update the doc list in `CLAUDE.md`.
