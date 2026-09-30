# Bridle docs

The decisions in [[docs/proposal/decisions|decisions]] are the designer's.
Everything else is a recommendation to be argued with. The agent host
(`design/agent-host/`) is built and describes the code; the rest of `design/`
is designed and not yet built ([[docs/proposal/build-order|build order]]).

## Layout

```
docs/
  README.md          this file
  proposal/          why bridle exists, what is decided, what it's for, the build order
  context/           facts the design works within: the projects, the host, prior research
  design/            the design, one topic per file
    agent-host/      the daemon, API and agent host: built (v1)
  research/          bridle's own research reports (dated records)
  spikes/            finished spike briefs and findings (NN-*.md, dated records)
    open/            spikes not yet run, one ticket each
  tickets/
    open/            open tickets, one ticket each
    resolved/        answered, with a Resolution section saying where the answer lives
```

## Onboarding a project

In the project's git repo: `bridle init` (scaffolds `.bridle/config.toml` and the `.gitignore`
entries), then `bridle sync`, `bridle doctor`, `bridle serve`
([[docs/design/cli|cli]]). Real onboardings: [[docs/context/onboarding-data-contracts|data-contracts]].
Step by step on dalek or the NUC, with the machine config and tokens: [[docs/context/adding-a-project|adding a project]].

## How the folders work

**Design and proposal docs are named by topic** and change in place, like
specs. They describe the current design, not its history; git has that. They
have no IDs and no frontmatter. Link to them by path:
`[[docs/design/gates|gates]]`, or `[gates](docs/design/gates.md)` inside a
table, where a wiki alias's `|` would break the row.

**Research and spike docs are dated records.** They aren't updated when the
design moves on; the design docs cite them.

**Questions and spikes are tickets**, following
`workflow-instructions/ticket-conventions.md`:

- One file each, `<descriptive-tail>-<id>.md`, with a 4-character ID that
  never changes. Create them with `bridle ticket new "<title>"` (files the
  matching task too; `--no-task` skips it), or by hand with a fresh ID unique
  across the repo.
- `bridle ticket set <id> <field> <value>` edits the frontmatter and `bridle ticket check`
  verifies frontmatter, IDs, `needs`/`see` and links; shipped to every project as the `tickets`
  rule (`workflow/base/rules/tickets.md`).
- The same frontmatter: `id`, `title`, `opened`, `repos`, `changes`, `specs`,
  `needs`, `see`. `needs:` orders them.
- The checker takes one ticket root, so `needs:` and `see:` only name tickets
  in the same tree (`questions/` or `spikes/open/`). A dependency across the
  two goes in the body as a `**Needs**` line with the ID and a link.
- **The folder is the state.** Resolving a question is `bridle ticket resolve <id>`
  (stamps `closed:` and moves `open/` to `resolved/`; you commit), plus a `## Resolution` section naming the design doc the
  answer landed in. The design doc, not the ticket, is the durable record. A
  spike's ticket moves to `spikes/done/` once its findings doc exists beside
  the numbered spike docs. (Whether bridle's own task records should keep folder-as-state
  is open: [[ticket-state-without-moving-files-p2ys|ticket state without moving files]].)
- Link to tickets by full stem, never a bare ID:
  `[[where-the-single-orchestrator-lives-hj4g|where the orchestrator lives]]`.
- Bodies quote their source verbatim (the doc section, the human's words,
  pinned to a commit when the source has since changed) and don't triage.

**Since P0-6 (tskm), `bridle task` is the queue, not this folder.** Every
ticket here has a matching bridle task (body starts `original id: <id>`); a
new ticket gets its task at filing, and resolving a ticket closes its task.
The ticket file stays the durable text its task points to — filing and
resolving still work the way this section describes. This doesn't decide
the folder-as-state question itself
([[ticket-state-without-moving-files-p2ys|ticket state without moving files]]),
which stays open.

Check links and IDs with:

```bash
for t in docs/tickets docs/spikes/open; do
  python3 ../workflow-instructions/scripts/check-tickets.py --root . --tickets $t --quiet
done
```

## Document index: which doc covers what

| Topic | Document |
|-------|----------|
| daemon, API, agent host | `docs/design/agent-host/*.md` (operating model, daemon, agents, messages, principals, API, roles and config) |
| CLI commands and flags | `docs/design/cli.md` |
| storage, database, state | `docs/design/storage.md` |
| git branches, release branches | `docs/design/agent-host/operating-model.md` or the project's `[branches]` config |
| roles and what they do | `workflow/base/roles/<role>.md` (worker, manager, product-manager, orchestrator, advisor) |
| building and testing | `CLAUDE.md` (Conventions section) |
| incidents (a task kind plus a broadcast notice) | `docs/design/agent-host/incidents.md` |
| permissions, tools | `docs/design/agent-host/principals.md` |
| email bridge (`bridle mail run`) | `docs/design/mail.md` |

## Reading order

1. [[docs/proposal/problem|The problem]], [[docs/proposal/decisions|decisions]],
   [[docs/proposal/goals-and-non-goals|goals and non-goals]],
   [[docs/proposal/build-order|build order]].
2. [[docs/context/projects|The projects]] and [[docs/context/nuc-host|the host]].
3. [[docs/design/overview|Architecture overview]].
4. The agent host, which is what runs today:
   [[docs/design/agent-host/operating-model|operating model]],
   [[docs/design/agent-host/daemon|daemon]],
   [[docs/design/agent-host/agents|agents]],
   [[docs/design/agent-host/messages|messages]],
   [[docs/design/agent-host/principals|principals]],
   [[docs/design/agent-host/api|API and events]],
   [[docs/design/agent-host/orchestrator-supervision|orchestrator supervision]] (not built),
   [[docs/design/agent-host/roles-and-config|roles and config]],
   [[docs/design/cli|CLI]].
5. The rest of the design as needed:
   [[docs/design/roles-and-lifecycle|roles and lifecycle]],
   [[docs/design/coordination|coordination]],
   [[docs/design/storage|storage]],
   [[docs/design/usage-and-budget|usage and budget]],
   [[docs/design/workflow-layers|workflow layers]],
   [[docs/design/components|components]],
   [[docs/design/gates|gates]],
   [[docs/design/knowledge-tiers|knowledge tiers]] and the tiers under it,
   [[docs/design/specs|specs]], [[docs/design/impact-and-conflicts|impact]],
   [[docs/design/specs-to-tests|specs to tests]], [[docs/design/spec-flow|the spec flow]],
   [[docs/design/worktrees-and-ports|worktrees and ports]],
   [[docs/design/skills|skills]].
6. `tickets/open/` and `spikes/open/` for what isn't settled.

For a plain-language read of one part as built today, see [[docs/briefs/README|the briefs]].
