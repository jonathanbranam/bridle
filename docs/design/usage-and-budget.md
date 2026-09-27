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
  sessions**. Bridle must never be what hits one: idle bridle work is always
  the better outcome ([[docs/design/usage-and-budget#The budget governor|budget governor]]).
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
| status line JSON | `rate_limits.five_hour` / `.seven_day`: `used_percentage`, `resets_at`, plus session cost and context use (field names not confirmed against Claude Code's own docs; parsed tolerantly, [[docs/design/cli#Built|cli.md]]) | interactive sessions (the human's, the orchestrator): bridle ships `bridle statusline` as the status line command, which shows the numbers **and** records them |
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

## What bridle records today

Built with the agent host, for the agents it hosts:

- every `turn.ended`'s four token counts and its cost, per agent and turn
  ([[docs/design/storage#The daemon's database|database]]), kept after the
  agent is removed. The cost counter
  is cumulative per session and survives `--resume`, so a turn's cost is the
  difference between consecutive counters;
- the latest `rate_limit_event` per window. It arrives once per process, so
  it can be stale while no agent is starting.
- `bridle statusline` snapshots from interactive sessions bridle doesn't host: the same
  rate-limit windows (via the same `upsert_rate_limit` path), plus a per-invocation row
  (cost, context use) in its own table, since there's no agent to attach it to.

`bridle usage` (and `GET /v1/usage`) shows per-agent turns, tokens and cost,
totals, the cache hit ratio (cache reads ÷ all input tokens), the last
known utilisation and reset time per window, and today's `bridle statusline`
rows. A role can also cap each agent's spend ([[docs/design/agent-host/agents#Spend cap|spend cap]]).

`bridle usage --by role|model|agent --since <duration>` (and
`GET /v1/usage/breakdown?since=&by=`) aggregates the turns ledger instead:
turns, tokens, cost and the cache hit ratio, grouped by role or model across
every agent that shares one, or per agent (the same grouping `bridle usage`
shows by default, but read from the turns ledger so `--since` applies to it
too). `--since` accepts a plain `<n><unit>` duration (`s`/`m`/`h`/`d`, e.g.
`30d`) and keeps only turns started within it. Task, project and kind
grouping, and `bridle usage task`/`trend`/`compare` and `bridle cost audit`
below, are not built: they need the ledger's task, kind and
workflow-revision columns, which don't exist yet (below).

Not built: the full usage ledger's task, role and workflow-revision columns.
The governor is built: it computes
`normal`/`holding`/`winding_down`/`paused` from `hold_at`/`wind_down_at`/
`stop_at` threshold crossings (per default-scoped window and, separately,
per per-model window), from `rejected`/`allowed_warning` rate-limit
statuses, and from reading staleness while any agent is working, and it
polls the undocumented `get_usage` control request
([[docs/design/usage-and-budget#Seeing the windows|seeing the windows]]) on
a running agent or a dedicated probe process. It holds new work while not
`normal`: `bridle spawn`/`resume` are refused (409) unless `--ignore-budget`
is passed, and a message that would start a turn in an idle agent is held
instead. On crossing into `winding_down` or worse, it stops idle agents at
once, sends working ones the usage-pause notice and stops them with exit
reason `budget_paused` when their turn ends or `wind_down_grace` expires,
and resumes paused agents (up to `max_workers`, oldest-paused first) once
every window is back below `resume_below` and no human hold is in force.
`bridle budget hold`/`release` (and `POST /v1/budget/hold`/`release`) drive
the same wind-down for a human-requested pause, for the current daemon only
(see the open question below). `budget.state` events, `agent.exited` with
reason `budget_paused`, `GET /v1/budget` and `bridle budget` show the
state.

## The budget governor

**Bridle never takes the account to a limit.** Hitting one blocks the human's
own Claude use (here, on the laptop, on the phone) until the window resets,
and that is far worse than any amount of bridle work waiting. So the governor
is built to leave bridle idle rather than risk it: it stops starting work
early, winds running agents down before the limit, and treats not knowing the
numbers as a reason to stop. Throughput is the thing it gives up.

### Thresholds

Per window, as a percentage of the window used, account-wide:

```toml
[budget]
max_workers  = 2            # concurrent headless agents

# per window; `default` applies to any window not named
hold_at      = { default = 80 }               # start nothing new
wind_down_at = { default = 90, seven_day_opus = 85 }   # tell every agent to wrap up and stop
stop_at      = { default = 95 }               # interrupt and stop at once, no wrap-up turn
resume_below = { default = 70 }               # every window must be below this to resume
wind_down_grace = "5m"      # how long a wrap-up turn may run before it's interrupted
max_staleness   = "10m"     # older readings count as unknown
```

- **`hold_at`**: no spawns, no resumes, no autostarts, and no message that
  would start a turn in an idle agent (it stays `pending`). Agents already
  working carry on.
- **`wind_down_at`** is the main knob, and the one to tune: how much of the
  window the human keeps for themselves is roughly `100 − wind_down_at`, less
  what the wrap-up turns spend. See [[docs/design/usage-and-budget#The wind-down|the wind-down]].
- **`stop_at`** is the backstop for a window moving faster than the wind-down
  can finish: every agent is interrupted and stopped straight away.
- **`resume_below`** gives hysteresis, so a reading that jitters around a
  threshold doesn't pause and resume the workforce repeatedly.
- **Per-model windows** (`seven_day_opus`, `seven_day_sonnet`) apply only to
  agents on that model; the others carry on. The rest apply to everyone.
- Thresholds are **account settings**, so they live in `~/.bridle/config.toml`
  and every daemon on the machine uses the same ones. A project's
  `.bridle/config.toml` may lower them, never raise them.
- **Unknown is not safe.** With no reading newer than `max_staleness` while
  any agent is working, the governor holds. Once the reading is three times
  that old, it winds down. Before the daemon's first reading ever lands, its
  own age stands in for the reading's age, so a fresh daemon doesn't hold
  before its first `get_usage` poll has had a chance to answer.

### The wind-down

When any window crosses `wind_down_at`, or the human asks for a hold:

1. The governor enters `winding_down` and emits `budget.state`.
2. **Idle agents are stopped at once** (closing stdin costs nothing). A message
   would start a turn, so they get no notice.
3. **Working agents get a notice** from `bridle`, sent `now` so it folds into
   the current turn at the next tool boundary:
   `Usage pause: <window> is at 91%. Commit your work in progress to your
   branch, send your manager one line on where you are, and end your turn.
   Don't start anything new.` Every role's preamble carries a one-line rule
   for this notice, so the notice itself stays short and the prompt cache
   holds.
4. **Every other message to an agent is held** as `pending`, so nothing starts
   a new turn.
5. When a notified agent's turn ends, bridle stops it with exit reason
   `budget_paused`. After `wind_down_grace`, or at once past `stop_at` or on a
   `rejected` rate-limit event, it's interrupted and stopped instead.
6. The governor is then `paused`. Each paused agent keeps its session id,
   worktree, branch and pending messages, as any `stopped` agent does. Once
   tasks exist, its task moves to `paused:limit` and keeps its claim.

The manager and orchestrator bridle hosts are agents like any other and wind
down the same way.

### Resuming

When **every** window is below `resume_below` and no human hold is in force,
the governor resumes paused agents with `--resume <session-id>`, in priority
order, up to `max_workers`. Each gets its pending messages, or, if it has
none, one line saying the pause is over. Then the manager is told. Short of
that, bridle stays idle: a paused `seven_day` window can mean days without
bridle work, and that is intended.

`bridle spawn` and `bridle resume` are refused (409, naming the window and its
reset time) while the governor holds or is paused; the human can pass
`--ignore-budget` to run one anyway.

### The human's hold

The human can idle bridle whenever they want the account for themselves:

```
bridle budget                       # windows, thresholds, governor state, what's paused
bridle budget hold [--for 3h | --until 18:00]
bridle budget release
```

`hold` does exactly what crossing `wind_down_at` does, and is meant to apply
to every daemon in the registry; today it only reaches the current daemon
(`POST /v1/budget/hold` on the daemon the CLI is already talking to). Reaching
every other project's daemon needs a cross-project credential the CLI
doesn't have yet:
[[how-does-bridle-budget-hold-identify-itself-to-other-daemons-hb0q|how does
`bridle budget hold` identify itself to other daemons]]. Without `--for` or
`--until` it lasts until `release`.

### Seeing the windows

The governor needs a **fresh** reading, and the stream doesn't give one: a
`rate_limit_event` arrives once per process (spike 01 S8), so a long-running
agent's last reading can be hours old, and it can't see the human's own use.
Sources, best first:

1. **`get_usage` probes.** The control request returns every window's
   utilisation and reset time **without a model call** (~1.1 s, spike 01). The
   daemon sends it to a running agent, or to a probe process of its own
   (`claude -p`, stream-json, no prompt, no tools) when none is running, and
   records only the window numbers (the response also carries account
   details). It polls every 5 min below `hold_at` and every 30 s above, and
   whenever a turn ends above `hold_at`. The reading is account-wide, so it
   includes the human's sessions on every machine.
2. **`rate_limit_event`s** from agents, and `bridle statusline` snapshots from
   the human's interactive sessions, as they arrive.
3. **An estimate** between readings: the ledger's tokens since the last
   reading, converted with the learned tokens-per-percent. It only ever makes
   the governor more cautious, never less.

`get_usage` is undocumented, so the contract suite covers it, and if it stops
working the governor falls back to 2 and 3 and holds at the staleness limits
above. `allowed_warning` and `rejected` events trip the wind-down and
`stop_at` respectively, whatever the percentages say; when Claude sends
`allowed_warning` is unknown ([[usage-probe-and-wind-down-headroom-u7pw|spike u7pw]]).

### Model choice

```toml
[models]                            # defaults by role; the governor may step down
manager  = ["opus", "sonnet"]
planner  = ["opus", "sonnet"]
reviewer = ["sonnet", "opus"]       # opus for protected/arch-revision reviews only
worker   = ["sonnet", "haiku"]
chore    = ["haiku"]
explore  = ["sonnet"]
```

New work starts from the role's list and steps down as a window gets tight,
e.g. Sonnet instead of Opus when `seven_day_opus` is high. A task can pin a
model (`model = "opus"`) when its plan says the step-down would be a false
economy. Stepping down delays a wind-down; it never replaces one.

### Across projects

Each project's daemon runs its own governor. They agree on when to pause
without talking to each other, because the readings are account-wide and the
thresholds are machine-wide. Sharing `max_workers` between them is still open:
[[how-project-daemons-share-one-budget-xypj|how project daemons share one budget]].

### API

`budget.state` (`{from, to, window, utilization, resets_at, reason}`, `to` one
of `normal`, `holding`, `winding_down`, `paused`) on each transition;
`agent.exited` with reason `budget_paused`; the governor's state in
`GET /v1/status`; and `GET /v1/budget`, `POST /v1/budget/hold`,
`POST /v1/budget/release`. All built, current-daemon-only per the hold gap
above.

## Designing for fewer tokens

Rules the design follows, and that the build is reviewed against:

1. **Bookkeeping is done by the tool, not the model.** Conflict detection,
   merges, trace links, spec parsing, rendering, state transitions and git
   commits are Rust code. No agent ever hand-edits bridle's state or reconciles
   it.
2. **The system prompt is stable, and the task is in the first message.**
   Everything role-specific and slow-changing goes in
   `--append-system-prompt-file`, identical for every agent in that role and
   project up to a short identity sentence (name, cwd, branch) appended last;
   that keeps the long, role-scoped part of the prefix cached across agents.
   The task-specific content goes in the first user message.
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
bridle usage --by role|model|agent --since 30d   # built: role/model/agent, no project or kind yet
bridle usage task tw-7fa2         # what one task cost, per agent and phase
bridle usage trend --per kind     # tokens per task kind over time
bridle usage compare --workflow <rev-a> <rev-b>   # did a workflow change cost more?
bridle cost audit [--check]       # static: size of everything bridle injects
```

`--by project` and `--by kind`, and everything below this line, need the
ledger's task, project and workflow-revision columns, which don't exist yet
("What bridle records today" above). The last two of the built ones answer
*"are new systems increasing the token budget?"* directly:

- **`bridle cost audit`** counts the tokens bridle adds to each role's context
  with no work done: prime, the rendered system-prompt file, skill
  descriptions, MCP tool schemas and hook boilerplate. Built: only the
  rendered system-prompt file exists in bridle today, so that's all it
  measures — specifically `stable_system_prompt`'s output, the role-scoped
  part meant to be identical across agents of a role, from a fresh render
  per role. The other categories are unbuilt (prime, skills) or deferred
  (MCP, hooks); `bridle_daemon::cost_audit::measure` folds each in once it
  exists, rather than reporting a placeholder zero for it now. It compares
  the result with a committed baseline (`.bridle/cost-baseline.json`, a
  record, not generated output — a human or worker writes it deliberately
  when moving the baseline). Since there's no tokenizer dependency in the
  workspace and this count is only a relative unit (see the caveat on
  `total_cost_usd` above), sizes are approximated at ~4 bytes/token.
  `--check` fails when a role's current size exceeds its baseline by more
  than `bridle_daemon::cost_audit::GROWTH_THRESHOLD_PERCENT` (10% by
  default). The growth then shows up in review, as a number, before it costs
  anything.
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
