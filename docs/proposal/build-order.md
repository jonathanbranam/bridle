# Build order

> **Status (checked 2026-10-03):** Built and in use: v1, P0, P1's messaging and stop-check,
> landing, the budget governor, the TUI, meta-notes on bridle specs, track-web's trial · Built, not wired
> in: packs and project rules until each daemon runs 9561950 or later, `sync`, layer hooks,
> `bridle wait`, impact and conflicts, traceability, ports, paired worktrees, the vitest
> adapter, the prototyper role, the gateway · Planned: component nesting, the harness and
> file-db migrations, the P8 web UI itself, take/give, permission prompts, MCP.

Each phase is usable on its own. Bridle's own development is the first test
project. Every phase sits on the daemon and API that v1 built.

**What the status words mean.** **Built and in use**: in the code and something in the normal
flow (a role prompt, the daemon, a project's config) uses it. **Built, not wired in**: the code
exists and is tested, but nothing in the normal flow invokes it, so it has never changed what an
agent does. **Planned**: design only. "Built" alone is never enough to rank work on: check
"in use" against the code first (ticket
[[the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw|34bw]] is what happens
when that's skipped). A change merged to `main` reaches agents only once each project's daemon
is restarted on a binary built from it.

| Phase | Delivers | Proves | State |
|---|---|---|---|
| **v1** | the agent host: daemon per workspace, HTTP/SSE API, CLI, spawn/message/observe/control headless agents, provenance ([agent host](docs/design/agent-host/operating-model.md)) | bridle can run and supervise a workforce | **built and in use** |
| **P0a** | `bridle statusline` + usage ledger + `bridle usage` for sessions bridle doesn't host | a usage baseline for the **current** workflow, before anything changes ([usage tracking](docs/design/usage-and-budget.md)) | statusline and `bridle usage`: **built and in use**. The ledger for unhosted sessions: **built, not wired in**: the statusline stopped reporting to it (s8kn; `crates/bridle/src/commands/hook.rs:5-11`), and `interactive_usage` holds one empty row from 2026-09-28. The baseline wasn't taken |
| **P0** | tasks, edges, `ready`, claims with leases renewed by agent activity, state branch persistence, `rebuild` | the store and the git story | **built and in use** (`rebuild` is a recovery command, tested, never yet needed) |
| **P1** | messages to tasks and roles, `ask/answer`, `wait`, `prime`, stop-check | the test-config case: a manager waits for a worker, which asks a question, gets an answer and finishes | messages to agents, roles and tasks (`send --task`), `ask/answer`, stop-check (the Stop hook passed at spawn, `config.rs:346`): **built and in use**. `prime`: **built and in use** for orchestrator and advisor sessions (`crates/bridle/src/session.rs:44`), **built, not wired in** for spawned roles (no role prompt runs it; its rules part goes into the spawn prompt from 9561950, below). `bridle wait`: **built, not wired in** (no role or rule uses it) |
| **P2** | layers: `bridle-workflow` repo, packs, project overrides, `rules explain/diff`, `sync` rendering, the six skills | one workflow across two real projects (suggest otters + data-contracts: most different, least OpenSpec risk) | The layer files (in-repo `workflow/`, not a separate repo), packs (`python`, `typescript`, `vim`), project rules and `rules explain/diff`: built. Resolved rules reached **no spawned agent** until 9561950 (2026-10-03, 34bw step 1): before it the spawn prompt was the preamble plus one role file. As of this check no running daemon has that commit, so packs and project rules are still **built, not wired in** in practice ([34bw](docs/tickets/open/the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw.md)). `sync`: **built, not wired in** (nothing runs it; rendered skills are gitignored). Layer hooks (`arch-guard`): **built, not wired in** (34bw step 3, approved). Skills: 2 of 6. Component nesting for delivery: **planned**. P2's test (an override in one project changes what that project's agents do) hasn't passed yet |
| **P3** | parser for goals, architecture and specs; ids; `spec check/export`; `import openspec`; the explore task kind and its prime rules | data-contracts on bridle specs, with the Python adapter replacing `spec-to-feature.py` | parser, ids, `spec check/export/import`, the Python adapter: **built and in use** on meta-notes (specs imported in `15bfb81`, `spec check --require-ids` in its check command, the adapter vendored). data-contracts isn't on bridle (a prepared `bridle-adopt` branch, no daemon) |
| **P4** | impact registry, conflict protocol, post-merge rebase notices; trace links with suspect tracking; `arch-revision` → `re-evaluate` flow | two workers on one capability at once; an architecture change traced to the specs it affects | `task impact`, `task conflict`, `trace`, `arch`, the landing guard and `re-evaluate` tasks: **built, not wired in**. No role tells anyone to declare impact (the conflicts table is empty), and no project has `design/architecture/` or trace links. Post-merge "main moved" notices: **built and in use** |
| **P5** | worktree layouts (incl. paired), port registry, merge-tree probes, integration branch, the integrator | harness + track-web in parallel | per-agent worktrees, the integration branch, `task land` (the integrator) and its merge probe: **built and in use**. Paired worktrees and the port registry (`bridle port`, no allocation yet in any project): **built, not wired in** |
| **P6** | TS test adapter; migrate track-web, harness, meta-notes, file-db | everything on bridle | vitest adapter (`workflow/packs/typescript/adapters/vitest-bridle/`): **built, not wired in** (track-web hasn't imported specs). meta-notes on bridle (integration `main`): **built and in use**. track-web: a trial on `bridle-adopt` since 2026-09-29, in use. harness, file-db: **planned** |
| **P7** | prototyper role (workflow/base/roles/); strong guidance: prototype prompt is the whole brief; prototypes differ significantly | agents can focus on constraints without rethinking from existing design | role and its prompt (br-a4ea, `cacab9c`): **built, not wired in** (never spawned in any project; no role says when to use it) |
| **P8** | web UI for the human: to-dos and decisions to run through and check off; TUI preserved | the human can manage their workflow from the web | the gateway API (`bridle gateway`, 8 of 10 tasks): **built, not wired in** (not configured or running). The UI (the `bridle-ui` repo, br-1665): **planned**, being started |

Don't start at P5. Research 09 §7 still applies.

Agent-host work that can land between phases, roughly in this order:

1. **Budget governor**: **built and in use**, before P0a
   ([budget governor](docs/design/usage-and-budget.md)): `get_usage`
   polling, hold at 80%, wind every agent down at 90%, stop at 95%, resume
   when every window is back under 70%, and `bridle budget hold`. V1 already
   runs agents with nothing but a per-agent spend cap between them and the
   account's limits, and hitting a limit blocks the human's own Claude use.
   Spike [u7pw](docs/spikes/open/usage-probe-and-wind-down-headroom-u7pw.md)'s
   cheap questions come first.
2. **`bridle take` / `give`**: **planned**, parked (research 01 §5.3): interrupt, close, hand the
   human `claude --resume <session>` in the worktree, resume headless after.
   Parked, 2026-09-29: the human said, "I don't see a strong need for take / give yet ... let's park it for now."
3. **Permission prompts as questions**: **planned**; deferred, nice-to-have, not
   near-term work; requirements to be refined later: `--permission-prompts
   host` plus `--permission-prompt-tool` naming an MCP tool bridle serves
   (spike 03: not a `can_use_tool` control request on the existing
   stdin/stdout channel), answered from role rules or turned into a
   `question` to the manager or human. The real mechanism needs bridle to
   serve an MCP tool ([spike 03](docs/spikes/03-permission-prompt-tool-findings.md)
   confirms this), so this item depends on item 5. Plan:
   [[docs/design/agent-host/messages#Permission prompts as questions|messages.md]].
4. **The TUI**, a crate on `bridle-api`: agents list, event tail, per-agent
   logs, inbox and reply. **Built and in use** (`crates/bridle-tui`).
5. **MCP server** at `/mcp` on the same listener, with tools mirroring the
   CLI: **planned**; deferred, nice-to-have, not near-term work; requirements to be
   refined later. Reaching it from claude.ai or mobile needs bridle on
   public HTTPS with auth. `bridle/mcp-1` has a parked, uncommitted-to-main
   branch with a working but untested read-side MCP server.
