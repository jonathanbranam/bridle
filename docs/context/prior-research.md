# Prior research and sources

The design cites earlier research by number, for example "research 09 §2.2".
**A bare "research NN" means the workflow research**, which lives outside this
repo in the sibling checkout `workflow/research/`. Bridle's own research and
spikes live in this repo under `docs/research/` and `docs/spikes/`, and are
always linked by path.

## Outside this repo

Paths are relative to the parent directory `bridle/` that holds all the
sibling checkouts.

| Where | What |
|---|---|
| `workflow-instructions/` | the markdown driver workflow and ticket system bridle succeeds. Its `ticket-conventions.md` is the model for `docs/tickets/` and `docs/spikes/open/` here |
| `workflow/research/` | the numbered research below |
| `workflow-tools/` | the Gherkin tooling that [[docs/design/specs-to-tests|specs to tests]] replaces |

| Research | File in `workflow/research/` |
|---|---|
| 01 | `01-current-workflow.md` |
| 02 | `02-beads.md` |
| 03 | `03-gastown.md` |
| 04 | `04-beads-rust.md` |
| 05 | `05-related-work.md` |
| 06 | `06-wheelhouse-wyvern.md` |
| 07 | `07-comparison.md` |
| 08 | `08-formulas-and-molecules.md` |
| 09 | `09-orchestration.md` |
| 10 | `10-specs-and-the-work-graph.md` |
| 11 | `11-sdd-workflows.md` |
| 12 | `12-repo-topology-and-scoped-rules.md` |
| 13 | `13-ticket-system.md` |
| 14 | `14-swarm-forge.md` |

## In this repo

| Doc | What |
|---|---|
| [research/01](docs/research/01-agent-runtime.md) | the agent runtime: spawning, hosting, watching and cleaning up headless Claude Code |
| [spike 01 brief](docs/spikes/01-stream-json-client.md) | the stream-json client spike, as briefed |
| [spike 01 findings](docs/spikes/01-stream-json-findings.md) | verified stream-json behaviour; cite it rather than assuming how `claude` behaves |
