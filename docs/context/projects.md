# The projects bridle has to serve

The layering ([[docs/design/workflow-layers|workflow layers]]) is justified by how different these are:

| Project | Stack | What makes it distinct | Tracking today |
|---|---|---|---|
| **track-web** | TS monorepo, 10 `client-*` apps, `packages/`, SQLite | Many unrelated clients in one repo (time, watch, games, trips, family…) — rules differ **per client**; dev ports in `packages/config/dev-ports.json`; browser verification on a disposable second instance | OpenSpec, 112 specs |
| **pi/harness** | TS, pi-based | Consumes track-web's `dungeon-engine` via a relative `file:` symlink → worktrees must be **paired siblings** (research 09 §2); never kill dev servers; reserved ports 4100–4300 / 5175–5177 | OpenSpec, 25 specs |
| **otters** | TS game (`otter-life` + others) | Simulation/rendering separation; CLI-driven testing with no browser; world generation | `br` (beads_rust) |
| **file-db** | Python now, TS planned | GitHub-backed store; two language bindings must behave the same | OpenSpec, 0 specs |
| **data-contracts** | Python, uv | Spec→Gherkin→pytest-bdd pipeline; "no assistant memory — everything durable is git-tracked" | OpenSpec, 6 specs |
| **meta-notes** | Vimscript + Python | Vim test runner; bare-function pytest style; a plugin *and* a CLI | OpenSpec (Beads before that), 17 specs |

Two of the six have already tried a Beads tracker and one moved off it. That is
evidence the task-graph idea is wanted and that a tracker alone is not enough.
