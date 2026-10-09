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
claim_lease_after = "10m"          # a claimed task is released once its claimant has been
                                   # inactive this long (storage.md, claims)
self_upgrade = false               # build, drain and restart into a newer green main
                                   # commit, as `bridle restart --upgrade` (daemon.md, Upgrade)

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
tools            = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]   # the built-ins that exist at all (`--tools`)
disallowed_tools = []
system_prompt    = "workflow/base/roles/worker.md"     # appended; bridle adds its own preamble
max_budget_usd   = 3.0             # per process; see agents.md, Spend cap
stop_check       = true            # registers the `bridle stop-check` Stop hook (coordination.md);
                                    # true by default for `worker` and roles that start from it

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

- **Built-in roles** are `worker`, `manager`, `orchestrator`, `researcher`, `prototyper`, `designer` and `document-reviewer`
  ([[docs/design/roles-and-lifecycle|roles]]). The defaults are the values
  above; the `researcher` is the worker's defaults plus `WebSearch` and `WebFetch` in `allowed_tools` and `tools` (the worker has no web tools), reuses the worker's role prompt and adds a preamble sentence on citing URLs and reporting failed fetches; it exists in every project without a `[roles.researcher]`, and the manager picks it for web-research tasks; the orchestrator is like the manager without `Bash(git *)`; the `prototyper` starts from the
  worker's defaults (its role file, `workflow/base/roles/prototyper.md`, says to build only from the
  prototype prompt's constraints; a project's `.bridle/roles/prototyper.md` is appended to it, both in
  the agent's system prompt and in `bridle prime prototyper`); the `designer` likewise (`workflow/base/roles/designer.md`: read a problem ticket, write options and a recommendation into it, build nothing; append file `.bridle/roles/designer.md`; `bridle prime designer`); the `document-reviewer` likewise starts
  from the worker's defaults, with `workflow/base/roles/document-reviewer.md` (the comment format and a
  review round for one document; append file `.bridle/roles/document-reviewer.md`; `bridle prime
  document-reviewer`). None has a
  default `start_prompt` or `max_budget_usd`; the values
  above are examples. A role with no `system_prompt` uses
  `<workflow>/base/roles/<role>.md` when the project sets `workflow` and that file exists;
  an explicit `system_prompt` wins, and with no file there is no role prompt. After the role prompt, every agent's system prompt carries a `## Workflow rules` section: the role's resolved rules for the project (`Config::role_rules_text`, sharing `rules::rules_section` with `bridle prime`; components excluded). The interactive roles no daemon spawns (orchestrator, advisor, aide, prototyper, document-reviewer) get the same text as a `## Rules` section at the end of `bridle prime <role>`, which is their opening prompt (`bridle session` runs it). There is no `bridle prime manager`: the manager is daemon-spawned and has the section in its system prompt. If they can't be resolved (missing workflow dir, bad rule file) the section is omitted with a warning (a listed pack whose directory is missing just adds nothing; `bridle daemon doctor` flags it); a spawn, resume or restart never fails for it. Only the manager has `autostart = true` by default (so a
  project with no role config gets a manager at daemon start); `autostart = false` in
  `[roles.manager]` turns it off (pair it with `resume_on_restart = false`: otherwise an existing manager still resumes, and takes a costly turn, on every daemon restart), and any other role can set it on. Autostart skips a role that already has an agent, whatever its name. A role a project always needs (bridle's own `project-manager`) opts in with `autostart = true`; a role that already has an agent is never spawned again, and a budget hold refuses the autostart spawn. Bridle's own `.bridle/` has a working set. A project can (The role was called `product-manager` before ticket 9j2h; the daemon still reads the old name as `project-manager`, in config and for stored agents, and `bridle migrate` renames it in an existing project, see [[docs/design/migrations]].)
  override the built-ins or add roles, which start from the worker's defaults.
- **`disallowed_tools` defaults deny Claude Code's own built-ins that bypass
  bridle's coordination the same way as a direct `SendMessage` call would**
  (docs/tickets/resolved/agents-can-use-claude-codes-own-sendmessage-78sp.md):
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
- **`tools` removes tools; `allowed_tools` only permits them.** `tools` becomes
  `--tools`: an unlisted built-in's definition never enters the agent's context
  (about 6K tokens off a worker's first turn, docs/spikes/08-lean-context-findings.md).
  Defaults: `worker` `Bash,Read,Edit,Write,Glob,Grep`; `manager` the same
  without Edit/Write plus `Agent` (for `Explore` subagents; `Agent` is also in
  its `allowed_tools`, since `dontAsk` denies what isn't allowed). The
  orchestrator has none (full toolset) until its launch is measured. A
  project's `tools` **replaces** the default; `Skill` is left out of the defaults
  because the role prompts already carry the procedure and nothing tells an agent
  to invoke a skill (`.claude/skills/` is generated by `bridle sync`, untracked,
  so a worktree doesn't even have it); a role that lists `Skill` also
  gets `disableBundledSkills` so the bundled skills don't reload. Permissions
  still gate calls: a tool must be in both `tools` and `allowed_tools`.
- **Unknown sections and keys are warnings, not errors** (6hx4), so an older binary reads
  a file a newer one wrote. Each is reported as one line, per file: ``<file>: unknown section
  [<section>] (unknown to this build (bridle <version>); a newer build may use it)`` or
  ``<file>: unknown key <section>.<key> (...same...)``. A key within edit distance 2 of a
  known key of the same section gets ` - did you mean <known_key>?` (so `max_worker` is still
  caught). The daemon logs each finding (`warn`) at start-up and reload, saying nothing
  new when a reload finds the same set; the CLI prints each to stderr once per run;
  `bridle daemon doctor` lists them under "config warnings" and `--strict` makes them (and any
  other warning) exit 1. Wrong types and missing required keys are still errors.
  `bridle_api::config_warn` does it (the real structs, via `serde_ignored`); `credentials.toml`
  was always lenient. There is
  no `project` key: the project's name comes from its directory.
  Durations are an integer plus `s`, `m` or `h`. The config is read once, at
  start.
- **Every role's system-prompt file** is prefixed with a short bridle preamble.
  It says what bridle is, the agent's identity variables, a sentence about the
  built-in role, and how to use `bridle send`, `inbox`, `status` and `agents`
  with `--json`. A role's own prompt file may use `{{commands.check}}`,
  `{{commands.check_worker}}` (the worker's own gate: `commands.check_worker`, defaulting to
  `commands.check`; bridle's own project leaves it unset: the full `just check`),
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
  ([[docs/design/roles-and-lifecycle|roles]]). `bridle land <task> [--branch B]
  [--check-cmd CMD]` is that integrator: under one lock (one landing at a time) it
  squash-merges the branch (one commit: `<task id>: <title>`, summary body, `Task:`/`Branch:` trailers) in `<workspace>/integration` (a worktree on scratch branch
  `integrate/<task>` cut from the integration tip, created on first use), runs
  `[integration] check = "..."` there (unset: skipped, with a note; `--check-cmd`
  overrides; also skipped, with a note, when the integration branch is an ancestor of the
  branch tip, since the squash tree is then the tree the worker already checked; CI on `main`
  is the backstop). When a check that ran passes and its output has a nextest `Summary ... N tests run` line, N is
  sniff-checked against `<workspace>/last-full-test-count` (fail if 0, under half or over double the last count; with no
  last count only 0 fails) and then written there for workers to read, then moves the integration branch guarded by the tip it started from
  (`<branch> moved, retry` if it changed): `git update-ref`, or, when a worktree (the
  clone, say) has the branch checked out, `git merge --ff-only` there so its files follow
  (refused only if that worktree has uncommitted changes to a file the landing touches, which the error names; other dirty files are left as they are), and marks the task done with
  the merge commit. A conflict (probed first with `merge-tree`), a failed check, a moved
  tip, or a branch touching `design/architecture/**` for a task that isn't an
  `arch-revision` lands nothing (409). It never pushes. Events: `integrate.started`, `integrate.finished`
  (`{task, branch, ok, commit|error}`).
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

## Other keys

Also read from `.bridle/config.toml` (defaults in parentheses; each is documented where it acts):

- `[commands] check` (`"just check"`) and `check_worker` (unset: same as `check`): the
  `{{commands.*}}` substitutions above.
- `[integration] check` (unset): the `bridle land` gate above.
- `[integration] check_skip_paths` (empty): globs of paths the check can't see. If the integration
  branch moved during a landing's check by commits touching only these, `land` replays its squash
  on the new tip and lands without re-checking; any other move fails "moved, retry".
- `[integration] warm_build` (unset): background build after each land; see "Warm worktree `target/`".
- `[context] wind_down_at = { default = 200000, worker = 120000 }` (context tokens) and
  `wind_down_grace` (`"5m"`): an agent nearing its context limit is told to hand off, then
  renewed ([[agents#Renewing|agents.md]]).
- `[orchestrator] enabled` (`false`), `launcher` (unset: types `bridle session orchestrator --project <project>`; a value is typed into the pane verbatim; the old default string counts as unset), `relaunch_backoff` (`["30s", "2m", "10m"]`), `stable_after` (`"10m"`), `waiter_grace` (`"15m"`), `note_tokens` (`"150k"`), `plan_tokens` (`"180k"`), `handover_tokens` (`"200k"`; each at least the one before), `handover_deadline` (`"30m"`), `max_uptime` (`"12h"`): the orchestrator supervisor ([[orchestrator-supervision]]).
- `[sessions] warn` (`["150k", "200k", "250k", "300k"]`; four increasing counts) and `[sessions.advisor] warn` / `[sessions.aide] warn`: the context steps of interactive sessions: warn, plan a handover, the normal ceiling, the hard limit that forces a restart ([[orchestrator-supervision]], Interactive sessions).
- `[migrations] auto` (`true`): `bridle serve` applies the project's pending migrations at start-up; `false` leaves them to `bridle migrate`; see [[docs/design/migrations|migrations]].
- `[schedule] timezone` (`America/New_York`): the IANA zone a scheduled message is evaluated in when it names none.
- `[state] push` (`true`): push `bridle/state` to `origin` after flushes; set to `false` to opt-out; see [[storage#The state branch]].
- `[machine] check_interval` (`"30s"`), `load_per_core` (2.5): [[operating-model#Load watch|load watch]].
- `[disk] check_interval` (`"1h"`), `min_free_gb` (20): [[operating-model#Disk monitor|disk monitor]].
- `[tasks] settle` (`"5m"`; `0` turns it off): how long a task waits after creation or a human
  comment/edit before it can be claimed or listed ready. An invalid value warns, falls back to
  5m and is reported by `bridle doctor`; it never stops the daemon starting (coordination.md, "Settling").
- `[tasks] open_stale` (`"4h"`; `0` turns it off): a task left `open` (no activity on it) this long
  goes back to `pending` with a thread note saying why, and the orchestrator and whoever readied it
  are messaged; a task with an open question is left alone. Also, when a task is readied the
  daemon messages the running project-manager (else the orchestrator) once per burst, naming each
  task, unless the PM readied it (xz4f; `open_watch.rs`, checked on the settle-wake tick).
- `[tasks] prefix` (first two alphanumerics of the project name): task id prefix
  ([[../storage|storage.md]]).
- `workflow` (unset), `packs = []`: where the workflow layers live and which L2 packs to
  include ([[../workflow-layers|workflow layers]]); `workflow` also drives the default role
  prompts above.
- `workflow` precedence and paths: `~/.bridle/config.toml`'s `workflow` (the machine layer;
  `$BRIDLE_HOME/config.toml`) beats the project's `.bridle/config.toml`, since a project's
  path is written for one machine. A leading `~` and `$VAR`/`${VAR}` are expanded (an unset
  variable is an error); a relative path is taken against the repo. A workflow directory
  that is missing or unreadable is an error at `bridle serve`, `sync`, `prime` and `rules`,
  and a failing `bridle doctor` check ("referenced files"), never a silent empty base
  layer. A git url is an error (`WorkflowGitUrl`) at the same places: it is not resolved. One resolver: `Config::workflow_root`.
- `[components.<id>]` (`paths`, `parent`, `docs`, `consumers`, all optional): the component map
  ([[../components|components]]); a parent that isn't defined, or a cycle, is a config error.
- `[mail]`, `[gateway]` and `[interactions]` belong to `bridle mail run` and `bridle gateway`
  (their crates parse them); the daemon only accepts them, so adding one never stops a daemon
  starting.
- `[gateway] public_url` (machine config, optional per-project override in `.bridle/config.toml`):
  the bridle UI's base URL as the human opens it, e.g. `http://dalek.tailbc91f5.ts.net:7878`.
  `bridle link` builds ticket and task links from it; unset means no links.

## Focus hours

Ticket cvaq. `[[focus]]` periods in the machine `~/.bridle/config.toml` keep the human on their
real work. Same shape as `[[budget.schedule]]` (`days` a list or `"all"`, `start`/`end` as
host-local `HH:MM`), plus `name` and `mode`. A period belongs to the day it starts: with
`start < end` it is that day; with `end < start` it runs from `start` on a listed day to `end` the
**next** day, whatever that day is (so the part after midnight is matched against the previous
day's `days`; `sun` 23:00–07:00 covers Monday morning even if `mon` isn't listed). `start == end`
is empty. `[[budget.schedule]]` uses the same rule. The end a prompt, lock message or advisor
refusal reports is when quiet actually ends: periods of the same mode that touch or overlap are
followed to the last end (21:30–00:00 then 00:00–06:00 reports 6:00 AM), capped at 7 days when
periods cover the whole week.

An end before its start is written with `+1d` (`end = "06:00+1d"`: that time on the day after the
start day). `end = "00:00"` is the midnight ending the start day and needs no `+1d`, so
`21:30`–`00:00` is valid. The daemon is lenient on load: an end before the start without `+1d`
still means the next day and only logs a warning, so a surprising block never stops the daemon or
the budget governor. `bridle daemon doctor` is where it is an error (naming the block and the
fix, e.g. `night: end 08:00 is before start 23:00; write "08:00+1d"`); run it after any change
to `~/.bridle/config.toml`.

```toml
[[focus]]
name  = "work"
days  = ["mon", "tue", "wed", "thu", "fri"]
start = "08:00"
end   = "18:00"
mode  = "quiet"          # default; "locked" blocks prompts (see below)

# overnight example: one block covers the night (Sunday 21:30 to Monday 06:00, ...)
[[focus]]
name   = "sleep"
days   = ["sun", "mon", "tue", "wed", "thu"]
start  = "21:30"
end    = "06:00"
mode   = "quiet"
```

With no `[[focus]]` everything is off. In a `quiet` period `bridle focus gate` (see
[[cli]]) injects firm limits (at most 3 sentences or 60 words, the first a nudge back to work; no
extra tool calls, research, tickets, planning or threads; defer with "saved for <end> ET"; no
follow-up questions; restarting watchers such as wake loops and background tasks is always allowed); the hook ignores
prompts that are not the human's (`<task-notification>` prompts from finished background tasks:
no gate text, no block, no `prompts.jsonl` line; cc45); the advisor and orchestrator role text points at it as the source. A project opts out with `focus_hours = false`
in its `.bridle/config.toml`.

**Locked.** In a `locked` period the gate blocks every prompt, the orchestrator keeps running,
`bridle session advisor` / `bridle advisor start` refuse, and the daemon kills every advisor tmux
pane (`@bridle` = `advisor*`) from its stall-check loop, so a period that starts is caught within
one tick (`bridle_daemon::focus`). An active override lifts the lock the same way it silences the
nudge.

**Override.** The human can write `~/.bridle/focus-override.toml` by hand (no CLI command):

```toml
# Local time (machine's local zone, no offset)
until  = 2026-10-01T22:00:00
reason = "deploy is broken"

# Or UTC with offset or Z
# until  = 2026-10-01T18:00:00-04:00
# until  = 2026-10-01T22:00:00Z

# Or local time of day (next occurrence)
# until = "22:00"
```

`until` accepts three forms, all capped at 2 hours after the override takes effect:
1. Local datetime without offset (e.g., `2026-10-01T22:00:00`): interpreted in the machine's local
   zone, like `[[focus]]` periods.
2. Datetime with offset or Z (e.g., `2026-10-01T22:00:00Z`, `2026-10-01T18:00:00-04:00`): UTC-aware,
   parsed as-is.
3. String `"HH:MM"` (e.g., `"22:00"`): the next occurrence of that local time, today if still ahead,
   else tomorrow. A local time that does not exist or is ambiguous (DST gap or overlap) is rejected
   with a log line.

It takes effect only `focus_override_delay_minutes` (top-level in `config.toml`, default 10)
after the file was last written (its mtime), so tripping it is deliberate. (Not `[focus]
override_delay`: `[[focus]]` is an array of tables, so `[focus]` can't also exist.) While active,
the gate stays silent. A malformed file is logged and ignored. `bridle status` shows a `focus`
line while an override is pending or active. Agents can't write it: every role denies
`Edit`/`Write` of `~/.bridle/focus*` and `~/.bridle/config.toml` (`DENY_FOCUS_FILES`, and the
advisor/orchestrator session settings), and their role text says never to create or edit it. Not
built yet: quiet-noise routing, an override event in the catch-up summary.

**Prompt recording.** Ticket u6w9. The `bridle focus gate` hook appends one JSON line to
`~/.bridle/prompts.jsonl` on every prompt in interactive sessions (orchestrator and advisor),
recording the time, session id, role, machine hostname, and project name (no prompt text or other
content), with `"event":"prompt"`. The `Stop` hook `bridle focus reply` (both `bridle session`
roles) appends the same fields with `"event":"reply"` when the agent finishes answering, the other
end of the human's reading time. Lines from before `event` existed are prompts. The daemon serves
the file as `GET /v1/interactions` ([[api]]); the gateway turns it into time reports. The file grows one short line per prompt and is meant for later
reporting on the human's time. Failures (missing directory, read-only home, bad stdin, clock errors)
are silent: the hook still exits 0 and prints its normal gate output unchanged.

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
able to lower (docs/tickets/open/per-task-tools-and-model-k8dw.md).

## Per-spawn secrets

`bridle spawn <role> --env KEY=VALUE` (repeatable) sets an environment
variable in that one spawn's `claude` process only ([[../cli#Built|cli.md]]),
e.g. a paid API token (docs/tickets/resolved/per-task-secrets-and-network-access-2ty9.md).
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

The [[docs/design/workflow-layers|workflow layers]] are planned to replace the role
prompts later (not built: today the role prompt is a file, and the resolved rules are appended to it). The `[roles]` table stays as the place a role's own default model
and tools are set. `[models]` sits alongside it: an ordered, strongest-first
model list per role that the budget governor steps down through when a
spawn doesn't pin a model itself, e.g.
`worker = ["sonnet", "haiku"]` — see
[[../usage-and-budget#Model choice|Model choice]] for the defaults and the
step-down rule. A project may replace a role's list outright; a role with no
`[models]` entry falls back to its own `model` as a single-entry list.

## Warm worktree `target/`

`[worktrees] warm_target` (default `true`): after `git worktree add` for a worktree role,
macOS clones a prior build's `target/` into the worktree with `cp -cR` (APFS copy-on-write:
near-instant, no extra disk), so the first build is incremental. Elsewhere it does nothing. It
never fails a spawn (a missing `target/` or failed copy is a logged warning) and never writes to
the source. The source is the integration worktree's `target/` when it exists and no warm
build is running there (or it is newer than the clone's), else the clone's; the spawn logs which
and its age. Cargo fingerprints embed absolute paths, so some workspace crates still
rebuild; dependencies hit.

`[integration] warm_build` (unset: nothing runs) keeps the integration source fresh: after each
successful `bridle land` (including a skipped-check fast-forward) the daemon runs this shell
command, niced, in `<workspace>/integration`, in the background. Land returns first; at most
one build runs at a time and landings during a build queue exactly one more; a failure is
logged and never fails a land. This repo sets `cargo build --workspace --all-targets`.

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

`[worktrees] layout = "default"|"root"|"paired"` and `root = "/path/{task}"` choose where a new worktree
is created: `default` is `<workspace>/wt/<agent>`; `root` is an absolute template with
`{task}` (claimed task id, else agent name), `{agent}` and `{project}`, and needs `{task}` or
`{agent}`. Invalid roots are config errors; a resolved path inside the clone is refused at spawn.
`paired` takes the same `root` (the project's worktree goes at `<root>/<project>`) plus one or more
`[worktrees.pair.<name>]` tables (`path`, absolute; `mode = "worktree"|"symlink"`, default
`worktree`) for sibling repos placed at `<root>/<name>`.
See [[../worktrees-and-ports|Worktrees and ports]].

## Ports

`[ports] range = [4000, 4999]` (inclusive, the default) and `reserved = [..]` (default empty)
bound what `bridle port alloc` hands out; `range` with low > high is a config error. See
[[../worktrees-and-ports|Worktrees and ports]].
