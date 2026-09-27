# Roles and configuration

The daemon loads built-in defaults, then applies `<repo>/.bridle/config.toml`
over them. The `[budget]` section (the account-wide usage governor's
thresholds) is different: it's read from the machine-wide
`~/.bridle/config.toml` first, and a project's `.bridle/config.toml` may
only lower its four percentage thresholds, never raise them — see
[[../usage-and-budget#The budget governor|the budget governor]] for the
full shape and merge rule. Everything else below is project-scoped, as
usual:

```toml
[daemon]
listen      = "127.0.0.1:0"        # 0 = any free port; the chosen URL goes in daemon.json
stall_after = "10m"
stop_grace  = "30s"                # how long `stop` waits after closing stdin

[roles.worker]
model            = "sonnet"
effort           = "medium"
workdir          = "worktree"      # worktree | repo
base             = "HEAD"          # ref new worktrees branch from
permission_mode  = "acceptEdits"
allowed_tools    = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]
disallowed_tools = []
system_prompt    = ".bridle/roles/worker.md"     # appended; bridle adds its own preamble
max_budget_usd   = 3.0             # per process; see agents.md, Spend cap

[roles.manager]
model             = "sonnet"
workdir           = "repo"
permission_mode   = "dontAsk"
autostart         = false
resume_on_restart = true
allowed_tools     = ["Bash(bridle *)", "Bash(git *)", "Read", "Glob", "Grep"]
system_prompt     = ".bridle/roles/manager.md"
start_prompt      = "Check your inbox and tell the human you're ready."   # first message when spawned without one
```

- **Built-in roles** are `worker`, `manager` and `orchestrator`
  ([[docs/design/roles-and-lifecycle|roles]]). The defaults are the values
  above; the orchestrator is like the manager without `Bash(git *)`. None has a
  default `system_prompt`, `start_prompt` or `max_budget_usd`; the values
  above are examples. Bridle's own `.bridle/` has a working set. A project can
  override the built-ins or add roles, which start from the worker's defaults.
- **Parsing is strict**: an unknown key stops the daemon from starting.
  Durations are an integer plus `s`, `m` or `h`. The config is read once, at
  start.
- **Every role's system-prompt file** is prefixed with a short bridle preamble.
  It says what bridle is, the agent's identity variables, a sentence about the
  built-in role, and how to use `bridle send`, `inbox`, `status` and `agents`
  with `--json`. A missing prompt file is logged, not fatal. The preamble is
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

The [[docs/design/workflow-layers|workflow layers]] later replace the role
prompts. The `[roles]` table stays as the place models and tools are set.
