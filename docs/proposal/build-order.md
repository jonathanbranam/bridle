# Build order

Each phase is usable on its own. Bridle's own development is the first test
project.

| Phase | Delivers | Proves |
|---|---|---|
| **P0a** | `bridle statusline` + usage ledger + `bridle usage` (read-only, no other bridle features) | a usage baseline for the **current** workflow, before anything changes ([usage tracking](docs/design/usage-and-budget.md)) |
| **P0** | workspace, `project add`, tasks, edges, `ready`, claims with leases, state branch persistence, `rebuild` | the store and the git story |
| **P1** | `send/ask/answer/inbox`, `wait`, hooks (`prime`, `inject`, heartbeat, stop-check) | the test-config case: a driver waits for a worker, which asks a question, gets an answer and finishes |
| **P2** | layers: `bridle-workflow` repo, packs, project overrides, `rules explain/diff`, `sync` rendering, the six skills | one workflow across two real projects (suggest otters + data-contracts: most different, least OpenSpec risk) |
| **P3** | parser for goals, architecture and specs; ids; `spec check/export`; `import openspec`; the explore task kind and its prime rules | data-contracts on bridle specs, with the Python adapter replacing `spec-to-feature.py` |
| **P4** | impact registry, conflict protocol, post-merge rebase notices; trace links with suspect tracking; `arch-revision` → `re-evaluate` flow | two workers on one capability at once; an architecture change traced to the specs it affects |
| **P5** | `spawn` with worktree layouts (incl. paired), port registry, merge-tree probes, integration branch | harness + track-web in parallel |
| **P6** | TS test adapter; migrate track-web, harness, meta-notes, file-db | everything on bridle |

Don't start at P5. Research 09 §7 still applies.
