# Usage limits and token efficiency

## The constraint

**The ceiling is one $100/month subscription (Max 5x), with no API overage.**
That's the opposite end of the scale from Yegge's ~$2,800/month across 13
accounts (research 06 §5). Every design choice in bridle has to be judged by its
token cost as well as its function.

In practice this means:

- A subscription has **rolling limits, not a bill**: a 5-hour window, a 7-day
  window, and per-model weekly windows (`seven_day_opus`, `seven_day_sonnet`).
  Hitting one stops all work on the account, **including the human's own
  sessions**.
- Parallelism is not free. Two workers use the window twice as fast. The number
  of concurrent workers is a budget setting, not an architectural one.
- Tokens spent on coordination (priming, injected rules, tool schemas, agents
  maintaining the tracker) buy no work. Wheelhouse's *"agents burn tokens
  invisibly keeping beads synchronized"* is the failure this section exists to
  prevent.

## Where bridle can see usage

| Source | Gives | Available in |
|---|---|---|
| stream-json `rate_limit_event` | `status` (`allowed` / `allowed_warning` / `rejected`), `resetsAt`, `utilization` (0–1), and the window type (`five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet`). Emitted **when the status changes** | headless workers |
| stream-json `result` | `usage` (input, output, cache-creation and cache-read tokens), per-model `modelUsage`, `total_cost_usd` (list-price equivalent, **cumulative across turns** in streaming mode) | headless workers |
| status line JSON | `rate_limits.five_hour` / `.seven_day`: `used_percentage`, `resets_at`, plus session cost and context use | the interactive driver: bridle ships `bridle statusline` as the status line command, which shows the numbers **and** records them |
| assistant error `rate_limit` | a turn failed on a limit | both |
| OpenTelemetry | tokens, cost, per request | later, optional |

The status line matters more than it looks. It's how bridle sees the account's
windows while no workers are running, which means it can **measure today's
workflow before bridle changes anything** ([[docs/design/usage-and-budget#Tracking token use over time|usage tracking]]).

Caveats:

- `total_cost_usd` is a list-price estimate. For a subscription it's only a
  relative unit, useful for comparing things with each other.
- How many tokens a percentage point of a window corresponds to is not
  published and may change. Bridle **learns it** by relating the ledger to the
  window percentages over time, and treats that as an estimate.

## The budget governor

Bridle checks the budget before every dispatch:

```toml
[budget]
plan                 = "max-5x"
max_workers          = 2            # concurrent headless workers
reserve.five_hour    = 25           # % of the window kept free for the human
pause_at.five_hour   = 80           # stop dispatching
pause_at.seven_day   = 85
pause_at.seven_day_opus = 70

[models]                            # defaults by role; the governor may step down
driver   = ["opus", "sonnet"]
planner  = ["opus", "sonnet"]
reviewer = ["sonnet", "opus"]       # opus for protected/arch-revision reviews only
worker   = ["sonnet", "haiku"]
chore    = ["haiku"]
explore  = ["sonnet"]
```

- **Dispatch** checks headroom first. If a window is past its `pause_at`, bridle
  doesn't spawn. The ready queue waits in priority order, and explorations and
  `someday` work go last.
- **Model choice** starts from the role's list and steps down as a window gets
  tight, e.g. Sonnet instead of Opus when `seven_day_opus` is high. A task can
  pin a model (`model = "opus"`) when its plan says the step-down would be a
  false economy.
- **Pausing when a limit is hit**:
  1. On `allowed_warning`, bridle stops dispatching and lets running turns
     finish.
  2. On `rejected`, it interrupts cleanly at the next turn boundary.
  3. It records each paused agent's session id and moves its task to
     `paused:limit`. The claim, worktree and branch are all kept.
  4. It sets a timer for `resetsAt`.
- **Resuming** when the window resets: bridle restarts paused agents with
  `--resume <session-id>`, in priority order, up to `max_workers`, and tells the
  driver.
- **The human's reserve** is respected even when work is queued. The human's
  interactive sessions share the account, and running out mid-conversation is
  the worst outcome.

## Designing for fewer tokens

Rules the design follows, and that the build is reviewed against:

1. **Bookkeeping is done by the tool, not the model.** Conflict detection,
   merges, trace links, spec parsing, rendering, state transitions and git
   commits are Rust code. No agent ever hand-edits bridle's state or reconciles
   it.
2. **The system prompt is stable, and the task is in the first message.**
   Everything role-specific and slow-changing goes in
   `--append-system-prompt-file`, identical for every agent in that role and
   project. The task-specific content goes in the first user message. That
   keeps a long shared prefix cached across agents.
3. **Prime is role-scoped and has a size budget.** `bridle prime` has a token
   budget per role, e.g. a worker's prime ≤ 3k tokens. Guides are pointed to,
   not included, unless the task's components need them.
4. **Injections are silent by default.** Hooks print nothing when there's
   nothing new. Messages are summarised with a pointer, not pasted in full,
   when they're long.
5. **Tool schemas cost tokens on every turn.** The bridle MCP server exposes a
   small set of tools, and workers launch with `--strict-mcp-config` so they
   don't inherit every MCP server the human has configured.
6. **Output is terse.** Bridle's `--json` output is compact, and skills tell
   agents to use quiet or summary flags on test runners and linters.
7. **Cheapest adequate model per role** ([[docs/design/usage-and-budget#The budget governor|budget governor]]), with Opus reserved for
   planning, architecture work and protected reviews.
8. **Fresh context versus resuming.** A fresh worker re-reads its context; a
   resumed one carries its history forward. Bridle measures both ([[docs/design/usage-and-budget#Tracking token use over time|usage tracking]])
   instead of assuming one is cheaper.

## Tracking token use over time

Every `result` and status-line snapshot goes into a **usage ledger** in the
database. Each row records: time, project, task, task kind, role, agent,
model, the four token counts, the cost equivalent, turn count, and the
**workflow revision**, i.e. the bridle version plus the `bridle-workflow` git
revision in effect.

```
bridle usage                      # today / this window / this week vs limits
bridle usage --by role|project|kind|model --since 30d
bridle usage task tw-7fa2         # what one task cost, per agent and phase
bridle usage trend --per kind     # tokens per task kind over time
bridle usage compare --workflow <rev-a> <rev-b>   # did a workflow change cost more?
bridle cost audit [--check]       # static: size of everything bridle injects
```

The last two answer *"are new systems increasing the token budget?"* directly:

- **`bridle cost audit`** counts the tokens bridle adds to each role's context
  with no work done: prime, the rendered system-prompt file, skill
  descriptions, MCP tool schemas and hook boilerplate. It compares the result
  with a committed baseline (`.bridle/cost-baseline.json`, a record, not
  generated output). `--check` fails when a change to rules, skills or tools
  grows any role's fixed overhead by more than a set percentage. The growth
  then shows up in review, as a number, before it costs anything.
- **`bridle usage compare --workflow`** compares tokens per task kind before
  and after a workflow revision. The comparison is rough, because tasks
  differ, but a clear increase in "tokens per chore" after a rules change is
  exactly the signal needed.
- **Cache hit ratio** (cache-read ÷ total input) is reported per role. A drop
  means something broke the stable prefix in rule 2 above.

**Measure the baseline first.** `bridle statusline` and the ledger are
deliberately early in the build order ([[docs/proposal/build-order|build order]]), so the current OpenSpec workflow's
usage gets recorded before bridle replaces it. Without that baseline there's
nothing to compare against.
