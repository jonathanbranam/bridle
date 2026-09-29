# Roles and configuration

The daemon loads built-in defaults, then applies `<repo>/.bridle/config.toml`
over them. The `[budget]` section (the account-wide usage governor's
thresholds) is different: it's read from the machine-wide
`~/.bridle/config.toml` first, and a project's `.bridle/config.toml` may
only lower its four percentage thresholds, never raise them — see
[[../usage-and-budget#The budget governor|the budget governor]] for the
full shape and merge rule. `[budget]` can also carry `[[budget.schedule]]`:
named periods, in host-local days and `HH:MM` time-of-day (the daemon's own
`chrono::Local`, not a configured timezone), that replace the `five_hour`
thresholds while they're in effect — see
[[../usage-and-budget#Schedule|Schedule]]. The same lower-only rule applies
to a project's own schedule periods. Everything else below is
project-scoped, as usual:

```toml
[daemon]
listen      = "127.0.0.1:0"        # 0 = any free port; the chosen URL goes in daemon.json
stall_after = "10m"
stop_grace  = "30s"                # how long `stop` waits after closing stdin

[branches]
integration = "main"               # work merges here; worktree roles branch from here too
# release    = "dev"               # optional: only set for the dev+release pattern

[roles.worker]
model            = "sonnet"
effort           = "medium"
workdir          = "worktree"      # worktree | repo
base             = "HEAD"          # ref new worktrees branch from; "HEAD" resolves to
                                    # [branches] integration unless set explicitly
permission_mode  = "acceptEdits"
allowed_tools    = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]
disallowed_tools = []
system_prompt    = "workflow/base/roles/worker.md"     # appended; bridle adds its own preamble
max_budget_usd   = 3.0             # per process; see agents.md, Spend cap

[roles.manager]
model             = "sonnet"
workdir           = "repo"
permission_mode   = "dontAsk"
autostart         = true           # the built-in manager's default; false for every other role
resume_on_restart = true
allowed_tools     = ["Bash(bridle *)", "Bash(git *)", "Read", "Glob", "Grep"]
system_prompt     = "workflow/base/roles/manager.md"
start_prompt      = "Check your inbox and tell the human you're ready."   # first message when spawned without one
```

- **Built-in roles** are `worker`, `manager` and `orchestrator`
  ([[docs/design/roles-and-lifecycle|roles]]). The defaults are the values
  above; the orchestrator is like the manager without `Bash(git *)`. None has a
  default `system_prompt`, `start_prompt` or `max_budget_usd`; the values
  above are examples. Only the manager has `autostart = true` by default (so a
  project with no role config gets a manager at daemon start); `autostart = false` in
  `[roles.manager]` turns it off, and any other role can set it on. Autostart skips a role that already has an agent, whatever its name. A role a project always needs (bridle's own `product-manager`) opts in with `autostart = true`; a role that already has an agent is never spawned again, and a budget hold refuses the autostart spawn. Bridle's own `.bridle/` has a working set. A project can
  override the built-ins or add roles, which start from the worker's defaults.
- **`disallowed_tools` defaults deny Claude Code's own built-ins that bypass
  bridle's coordination the same way as a direct `SendMessage` call would**
  (docs/questions/open/agents-can-use-claude-codes-own-sendmessage-78sp.md):
  `SendMessage` and the `Workflow` tool, for every built-in role; `worker` and
  `manager` also deny the scheduling tools (`ScheduleWakeup`, `CronCreate`,
  `CronDelete`, `CronList`) and `RemoteTrigger`. The `Agent` tool is allowed
  since subagents run inside the same agent's process and cost counts toward
  that agent's budget, so it doesn't bypass bridle's coordination. The
  orchestrator keeps scheduling, since it paces its own loop with
  `ScheduleWakeup`, but still denies the rest. Unlike `allowed_tools`, a
  project's `disallowed_tools` is **additive**: it extends the role's built-in
  denials rather than replacing them, so a project never needs to re-list
  what's already denied by default to add one more.
- **Parsing is strict**: an unknown key stops the daemon from starting. There is
  no `project` key: the project's name comes from its directory.
  Durations are an integer plus `s`, `m` or `h`. The config is read once, at
  start.
- **Every role's system-prompt file** is prefixed with a short bridle preamble.
  It says what bridle is, the agent's identity variables, a sentence about the
  built-in role, and how to use `bridle send`, `inbox`, `status` and `agents`
  with `--json`. A role's own prompt file may use `{{commands.check}}`,
  `{{commands.check_worker}}` (the worker's own gate: `commands.check_worker`, defaulting to
  `commands.check`; bridle's own project sets `just check-affected`),
  `{{branches.integration}}` and `{{branches.release}}` (left as-is when unset),
  substituted at render time, so the base `worker`/`manager` prompts name no
  project's build tool or branch. A missing prompt file is logged, not fatal. The preamble is
  **identical for every agent of a role**, so the prompt cache holds
  ([[docs/design/usage-and-budget#Designing for fewer tokens|fewer tokens]],
  rule 2). Agent-specific facts (name, worktree path) go in the first user
  message, not the prompt file.
- **Every agent can reach bridle**: `Bash(bridle *)` is added to every role's
  allowed tools.
- **The manager runs in the clone** (`workdir = "repo"`) because it
  coordinates rather than edits. Merging is the integrator's job, and the
  integrator is bridle itself, in its own worktree
  ([[docs/design/roles-and-lifecycle|roles]]).
- **`[branches]` names the project's integration branch, and its release
  branch when it has one** — see [[../agent-host/operating-model#Branch
  pattern|operating-model.md, "Branch pattern"]] for the two supported
  patterns, the trial-onboarding case, and the mechanical `disallowed_tools`
  enforcement that keeps ordinary agents off the release branch. `integration`
  defaults to `"main"`, so bridle's own project needs no `[branches]` entry
  at all.

- **`[ci] github = true` turns on the CI watcher** (off by default; nothing is
  auto-detected). See [[../agent-host/operating-model#CI watcher|operating-model.md, "CI watcher"]].

- **`[messages] answer_for_human = [principal ids]`** names who may answer for the human
  (default `["external:orchestrator"]`): their `--reply-to` a message addressed to the human
  closes it. See [[messages]].

## Per-spawn tool overrides

`bridle spawn <role> --allow-tool TOOL` (repeatable) grants a tool beyond the
role's `allowed_tools` for that one spawn only ([[../cli#Built|cli.md]]).
It's per-agent, not per-role: nothing is written to `.bridle/config.toml` or
the role, so the next agent spawned with the same role gets the role's own
tools again. The grant is persisted on the agent's own record, though, so it
survives that one agent's `renew` (context handoff) and `resume` (daemon
restart) — both rebuild the `claude` command from the role plus this agent's
stored overrides, not the role alone. It only adds — there's no `--deny-tool`
to shrink a role's tools for one spawn, since `disallowed_tools` is meant as
a floor every agent of a role gets, not something a single spawn should be
able to lower (docs/questions/open/per-task-tools-and-model-k8dw.md).

## Per-spawn secrets

`bridle spawn <role> --env KEY=VALUE` (repeatable) sets an environment
variable in that one spawn's `claude` process only ([[../cli#Built|cli.md]]),
e.g. a paid API token (docs/questions/open/per-task-secrets-and-network-access-2ty9.md).
Same shape as `--allow-tool`: per-agent only, nothing written to
`.bridle/config.toml` or the role, so the next agent spawned with the same
role doesn't get it. Like `--allow-tool`, it's persisted on the agent's own
record and reapplied on that agent's `renew` and `resume`. It isn't logged or
returned by `bridle agents` or other read endpoints; only the variable names
are ever written to the daemon's log, never the values.

Network access for that secret needs no new mechanism: every role already
has `Bash` in its `allowed_tools` (see above), and `curl`/similar under
`Bash` already reaches any external API — no `--allow-tool` grant is needed
for that. `--allow-tool` matters for network access only when a narrower
tool than `Bash` is wanted, e.g. scoping `WebFetch` to one domain with
`--allow-tool 'WebFetch(domain:api.example.com)'`, if Claude Code's own
permission syntax supports scoping `WebFetch` that way.

The [[docs/design/workflow-layers|workflow layers]] later replace the role
prompts. The `[roles]` table stays as the place a role's own default model
and tools are set. `[models]` sits alongside it: an ordered, strongest-first
model list per role that the budget governor steps down through when a
spawn doesn't pin a model itself, e.g.
`worker = ["sonnet", "haiku"]` — see
[[../usage-and-budget#Model choice|Model choice]] for the defaults and the
step-down rule. A project may replace a role's list outright; a role with no
`[models]` entry falls back to its own `model` as a single-entry list.

## Warm worktree `target/`

`[worktrees] warm_target` (default `true`): after `git worktree add` for a worktree role,
macOS clones the clone's `target/` into the worktree with `cp -cR` (APFS copy-on-write:
near-instant, no extra disk), so the first build is incremental. Elsewhere it does nothing. It
never fails a spawn (a missing `target/` or failed copy is a logged warning) and never writes to
the clone's `target/`. Cargo fingerprints embed absolute paths, so some workspace crates still
rebuild; dependencies hit.

## Worktree setup command

`[worktrees] setup = "<shell command>"` (optional, unset = nothing runs): after `git worktree
add` and the `target/` warm-up, the daemon runs it with `sh -c` in each new worktree, for
worktree roles only, never the main clone (e.g. `npm install --prefer-offline`, so a Node
project's worker has `node_modules`). `setup_timeout_secs` (default 600) bounds it. The env is
the daemon's minus every `BRIDLE_*` variable. On non-zero exit or timeout the spawn fails with
an error naming the command, exit status and the last ~20 lines of output, and the worktree and
branch are removed as for any other failed spawn. Duration is logged at info. 

`[worktrees] copy = [".env", ".mcp.json"]` (optional, default empty) lists repo-relative
gitignored files a worker needs. After `git worktree add` and before setup runs, each one that
exists in the project clone is copied (not symlinked) to the same path in the worktree, keeping
its mode, so a 0600 token file stays 0600; parent directories are created. A missing file is
skipped with a logged warning and never fails the spawn. Absolute paths and paths containing
`..` are rejected when the config is parsed. Files only: no directories or globs.
