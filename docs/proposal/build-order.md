# Build order

Each phase is usable on its own. Bridle's own development is the first test
project. Every phase sits on the daemon and API that v1 built.

| Phase | Delivers | Proves | State |
|---|---|---|---|
| **v1** | the agent host: daemon per workspace, HTTP/SSE API, CLI, spawn/message/observe/control headless agents, provenance ([agent host](docs/design/agent-host/operating-model.md)) | bridle can run and supervise a workforce | **built** |
| **P0a** | `bridle statusline` + usage ledger + `bridle usage` for sessions bridle doesn't host | a usage baseline for the **current** workflow, before anything changes ([usage tracking](docs/design/usage-and-budget.md)) | **built** (statusline, usage ledger, bridle usage) |
| **P0** | tasks, edges, `ready`, claims with leases renewed by agent activity, state branch persistence, `rebuild` | the store and the git story | **built** (tasks, edges, ready, claims with leases, state branch, rebuild) |
| **P1** | messages to tasks and roles, `ask/answer`, `wait`, `prime`, stop-check | the test-config case: a manager waits for a worker, which asks a question, gets an answer and finishes | **partly** (messages to agents and tasks with send --task, ask/answer, stop-check built; wait and prime for non-orchestrator roles not) |
| **P2** | layers: `bridle-workflow` repo, packs, project overrides, `rules explain/diff`, `sync` rendering, the six skills | one workflow across two real projects (suggest otters + data-contracts: most different, least OpenSpec risk) | **as far as it goes** (packs, rules explain/diff, sync built; component nesting for delivery not) |
| **P3** | parser for goals, architecture and specs; ids; `spec check/export`; `import openspec`; the explore task kind and its prime rules | data-contracts on bridle specs, with the Python adapter replacing `spec-to-feature.py` | |
| **P4** | impact registry, conflict protocol, post-merge rebase notices; trace links with suspect tracking; `arch-revision` → `re-evaluate` flow | two workers on one capability at once; an architecture change traced to the specs it affects | |
| **P5** | worktree layouts (incl. paired), port registry, merge-tree probes, integration branch, the integrator | harness + track-web in parallel | plain per-agent worktrees built in v1 |
| **P6** | TS test adapter; migrate track-web, harness, meta-notes, file-db | everything on bridle | |

Don't start at P5. Research 09 §7 still applies.

Agent-host work that can land between phases, roughly in this order:

1. **Budget governor** **built**, before P0a
   ([budget governor](docs/design/usage-and-budget.md)): `get_usage`
   polling, hold at 80%, wind every agent down at 90%, stop at 95%, resume
   when every window is back under 70%, and `bridle budget hold`. V1 already
   runs agents with nothing but a per-agent spend cap between them and the
   account's limits, and hitting a limit blocks the human's own Claude use.
   Spike [u7pw](docs/spikes/open/usage-probe-and-wind-down-headroom-u7pw.md)'s
   cheap questions come first.
2. **`bridle take` / `give`** **parked** (research 01 §5.3): interrupt, close, hand the
   human `claude --resume <session>` in the worktree, resume headless after.
   Parked, 2026-09-29: the human said, "I don't see a strong need for take / give yet ... let's park it for now."
3. **Permission prompts as questions** — deferred, nice-to-have, not
   near-term work; requirements to be refined later: `--permission-prompts
   host` plus `--permission-prompt-tool` naming an MCP tool bridle serves
   (spike 03: not a `can_use_tool` control request on the existing
   stdin/stdout channel), answered from role rules or turned into a
   `question` to the manager or human. The real mechanism needs bridle to
   serve an MCP tool ([spike 03](docs/spikes/03-permission-prompt-tool-findings.md)
   confirms this), so this item depends on item 5. Plan:
   [[docs/design/agent-host/messages#Permission prompts as questions|messages.md]].
4. **The TUI**, a crate on `bridle-api`: agents list, event tail, per-agent
   logs, inbox and reply. **built** (`crates/bridle-tui`).
5. **MCP server** at `/mcp` on the same listener, with tools mirroring the
   CLI — deferred, nice-to-have, not near-term work; requirements to be
   refined later. Reaching it from claude.ai or mobile needs bridle on
   public HTTPS with auth. `bridle/mcp-1` has a parked, uncommitted-to-main
   branch with a working but untested read-side MCP server.
