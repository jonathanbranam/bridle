# Bridle docs

The decisions in [[docs/proposal/decisions|decisions]] are the designer's.
Everything else is a recommendation to be argued with. The agent host
(`design/agent-host/`), `design/cli.md` and `design/storage.md` describe the code as built
(planned parts are marked). The rest of `design/` mixes built and planned: each doc opens with a
`Status` line (built and in use · built, not wired in · planned), and what's next is in the
[[docs/proposal/build-order|build order]].

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

In the project's git repo: `bridle daemon init` (scaffolds `.bridle/config.toml` and the `.gitignore`
entries, and vendors the base workflow), then `bridle daemon doctor`, `bridle daemon serve`. Daemon-spawned
agents (not the orchestrator or advisor sessions) get their role's resolved rules (base, `packs`, the project's `.bridle/rules/`) in the system prompt
without any other step; `bridle workflow sync` (CLAUDE.md block, skills, hooks) is optional and only
matters if you commit its output
([[docs/design/cli|cli]]). Real onboardings: [[docs/context/onboarding-data-contracts|data-contracts]].
Step by step on dalek or the NUC, with the machine config and tokens: [[docs/context/adding-a-project|adding a project]].
Bringing a whole new machine onto the network (git, config, tokens, services, moving a project): [[docs/context/add-a-machine|add a machine]].

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
  never changes. Create them with `bridle ticket new "<title>" --kind <kind>` (files no task;
  once the ticket with its ask is committed on `main`, `bridle ticket task <id>` files it), or by hand with a fresh ID unique
  across the repo.
- `bridle ticket set <id> <field> <value>` edits the frontmatter and `bridle ticket check`
  verifies frontmatter, IDs, `needs`/`see` and links; shipped to every project as the `tickets`
  rule (`workflow/base/rules/tickets.md`).
- The same frontmatter: `id`, `title`, `kind` (a task kind; editable with `ticket set`),
  `opened`, `filed_by` (optional: the principal that filed it, e.g. `external:aide`, taken from the
  caller's identity by `ticket new`; older tickets have none and `ticket check` accepts that), `repos`, `changes`, `specs`, `needs`, `see`, `tasks` (every task made from the
  ticket; the task's `ticket` field is `<id>`, so the link is two-way and `ticket check`
  flags one side only). Tickets and tasks share one id alphabet and space: a ticket's first task
  takes its id (`br-<id>`), and whoever runs `ticket task` is the task's creator and so its watcher, told when it lands; `bridle ticket new --from-task <task-id>` goes the other way, making
  a ticket from a task with the task's id (a fresh, linked id when an old hex task id has `0` or `1`).
  Tickets hold design decisions (the why and the what), tasks the work and its status; a build that
  comes out of a discussion ticket gets its own feature ticket, linked, and its task is made from that. A missing `kind`/`tasks` is a warning until the backfill migration. `needs:` orders them.
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
ticket here has a matching bridle task (its `ticket` field is `<id>`); a
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
| `bridle report`, the daily "what happened" report | `docs/design/report.md` |
| `bridle docs` topic text (embedded in the binary) | `docs/cli/*.md` |
| storage, database, state | `docs/design/storage.md` |
| project migrations (`bridle migrate`) | `docs/design/migrations.md` |
| git branches, release branches | `docs/design/agent-host/operating-model.md` or the project's `[branches]` config |
| roles and what they do | `workflow/base/roles/<role>.md` (worker, manager, project-manager, orchestrator, advisor, prototyper, designer, document-reviewer) |
| building and testing | `CLAUDE.md` (Conventions section) |
| incidents (a task kind plus a broadcast notice) | `docs/design/agent-host/incidents.md` |
| the Windows PC as a bridle host under WSL2 (setup steps, planned) | `docs/context/windows-wsl2-host.md` |
| permissions, tools | `docs/design/agent-host/principals.md` |
| email bridge (`bridle mail run`) | `docs/design/mail.md` |
| human web UI (gateway built; the UI is a separate project) | `docs/design/human-web-ui.md` |
| throwaway dev site for verifying UI work (planned) | `docs/design/dev-site.md` |

## Releases

Pushing a `v*` tag runs `.github/workflows/release.yml`, which builds `bridle` and attaches to the
GitHub release for the tag (creating it if missing):

- `bridle-<tag>-<target>.tar.gz` for `x86_64-unknown-linux-gnu` (built on ubuntu-22.04),
  `aarch64-apple-darwin` and `x86_64-apple-darwin`; each holds just the `bridle` binary.
- `SHA256SUMS`, the `sha256sum` of every tarball.

Running the workflow by hand (`workflow_dispatch`) builds the tarballs and uploads them as
workflow artifacts without publishing. `scripts/package-release.sh` does the packaging.

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
   [[docs/design/agent-host/orchestrator-supervision|orchestrator supervision]],
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
