# The agent host: operating model

The agent host is the part of bridle that runs agents: a long-running daemon,
the API every client uses, and the supervisor that hosts headless Claude Code.
It is **built** (v1), and everything later (tasks, layers, specs, the
integrator) sits on the same daemon and API.

## What it does

Start bridle against a clone, in the foreground or as a background daemon.
Then drive it from a CLI that the human, the human's own orchestrator agent,
the agents bridle hosts, and later a TUI, GUI or MCP server all share:

- **spawn** headless Claude Code agents, each in its own git worktree (or the
  clone, or a given directory), with a role that sets model, prompt, tools and
  permissions;
- **message** them. Messages go to an agent, to the human's inbox, or from one
  agent to another, with delivery tracked;
- **observe** them: status, the live event stream, transcripts and usage;
- **control** them: interrupt, stop, resume after a crash or daemon restart,
  and remove them with their worktree;
- **record who did what.** Every action carries the principal that caused it:
  the human, a bridle-hosted agent, an external agent (the orchestrator) or
  bridle itself ([[docs/design/agent-host/principals|principals]]).

## The flow

1. **Create a workspace folder.** It's the daemon's home: it holds the clone,
   the worktrees and bridle's state ([[docs/design/agent-host/daemon|daemon]]).
2. **Clone the project's integration branch** into it (`main` by default —
   see "Branch pattern", below).
3. **Run `bridle serve` in the clone.** Bridle reads `<repo>/.bridle/config.toml`
   if present (roles, defaults); the file is optional. Bridle needs no worktree
   for itself: it never edits code, and its state is a SQLite file under
   `<workspace>/.bridle/`. Agents get worktrees under `<workspace>/wt/<agent>` (the default `[worktrees] layout`),
   on branches `bridle/<agent>`. The clone's own checkout is left to the human
   and the manager.
4. **"Start working" means starting the manager**: `bridle spawn manager`, or
   it autostarts with the daemon by default. Bridle is
   mechanism: it spawns, delivers, supervises, records and (later) integrates.
   Deciding what to work on is judgement, so it's the manager's job
   ([[docs/design/roles-and-lifecycle|roles]]). The work comes from the task queue
   (`bridle queue`, `bridle task ready`), which the project manager (or the orchestrator) fills.
5. **The orchestrator talks to bridle through the CLI**, with a token that
   identifies it as `external:orchestrator`. It never needs the repo: when
   bridle is remote, its only view of the code is through bridle and the
   agents it hosts. It is optional; bridle runs without it.
6. **The CLI works at any time, from a terminal, TUI, GUI or agent.** The CLI
   is an API client, so a TUI or GUI uses the API directly and gets the live
   event stream ([[docs/design/agent-host/api|API]]).
7. **Human actions are distinguished from agent actions.** Every API call is
   authenticated by a token that names a principal, and every event records it.
   On one machine, as one Unix user, this is *attribution, not security*
   ([[docs/design/agent-host/principals|principals]]).

**The channel is HTTP from day one**: JSON over HTTP for commands, Server-Sent
Events for the live stream. It listens on `127.0.0.1` by default. Listening on
another interface is one flag, which lets the workforce run remotely while the
orchestrator and TUI stay on the laptop.

## No-manager mode

A project can run without a standing manager: `[roles.manager]` with `autostart = false` and
`resume_on_restart = false`. The orchestrator starts one (`bridle spawn manager`) when the
human asks or there is work, and stops it when nothing is ready or running. Manager-bound notices
meanwhile: "task filed" goes to the orchestrator; CI failures on `main` go to the human and wake the
orchestrator; main-moved notes go to workers, not the manager; `main` moving doesn't wake the orchestrator
(pdmd: it needs no decision).

## Disk monitor

Decision (ticket m3wq): the daemon runs the check, on a timer, not a role polling, because it
must keep working when no manager or orchestrator is up and costs no tokens. Every
`[disk] check_interval` (default `1h`; `0s` turns it off) it reads free space on the workspace
volume and the allocated size of the clone's `target/`, `wt/` and `.bridle/`
(`bridle-daemon/src/disk.rs`; the walk runs in `spawn_blocking`). Each reading is logged at
`info` and recorded as a `disk.checked` event (`{free_bytes, total_bytes, target_bytes,
worktrees_bytes, data_bytes}`), so growth over time is queryable from the event log. Only a real
problem reaches the human's inbox ([[stop-status-notes-to-the-human-inbox-kp3f|kp3f]]): free
space under `[disk] min_free_gb` (default 20) sends one note with the sizes and a remediation
(`cargo clean`, remove finished worktrees), and not again until free space has recovered and
dipped again. Investigating growth is left to whoever reads the events.

## Load watch

Decision (ticket 58c9, first slice): the daemon samples the machine's 1-minute load average
every `[machine] check_interval` (default `30s`; `0s` turns the watch off) and divides it by the
core count (`bridle-daemon/src/load.rs`). It reads `/proc/loadavg`, else `sysctl -n vm.loadavg`:
`unsafe` is forbidden here, so no `getloadavg` call. The last reading is in `bridle status`
(`load`). The keys are read from the project's `.bridle/config.toml` (the machine file's `[machine]`
section holds other keys that the daemon doesn't read; moving these there is not yet done). While load per core is above `[machine] load_per_core` (default 2.5; zero or less never
holds) new spawns are refused with a conflict, like a budget hold (`--ignore-budget` skips it);
they resume on their own once the load falls. The orchestrator gets one note per crossing (not
per tick) with the load and the top three CPU consumers by command name (`ps`); its role says to
add no work and wait. Resumes and renews of existing agents are not held, and running agents are
not wound down (rejected for now: by-priority wind-down and cross-project coordination, design
xypj; killing processes; fixing the cause, a target-dir copy at spawn).

## CI watcher

`[ci] github = true` in `.bridle/config.toml` makes the daemon watch GitHub Actions for the
integration branch (`bridle-daemon/src/ci.rs`, a background loop like the governor's; it
shells out to `git` and `gh` from the main clone, so `gh` must be installed and
authenticated). Every minute-long tick it checks `git ls-remote origin <integration>` (every
third tick while idle, so it doesn't depend on seeing the merger's push). A new tip is polled
with `gh run list --commit <sha>` each tick until every run is `completed` (no runs yet keeps
polling); it gives up after about an hour. Then it emits `ci.completed` (`{sha, conclusion,
url}`; `conclusion` is `success`, `failure` or `cancelled`) and remembers the result for
`bridle status`. On `failure` only, it sends a note to the first running `manager` agent (the
human if none): "CI failed on <sha>: <failed jobs>; <url>. Don't merge until it's green." The
job names come from `gh run view <id> --json jobs`. A missing or failing `gh`/`git` is one
logged warning until the next success, and the next tick retries. The daemon's first tick
after a restart treats the current tip as new, so a red tip is reported again then.

## Branch pattern

A project names its branches with `[branches]` in `.bridle/config.toml`
(`bridle-daemon/src/config.rs`, `BranchesConfig`): one setting doubles as
both "where new work branches from" and "where completed work merges to and
pushes to", so there's a single knob, not two.

```toml
[branches]
integration = "dev"    # work merges here, agent worktrees branch from here (default "main")
release = "main"       # optional; unset means the trunk pattern (below)
```

Exactly two shapes, KISS (the human, 2026-09-28: "some projects work
directly on main, others use a dev branch... don't add complexity for other
approaches"):

- **Trunk** (`release` unset, bridle's own pattern): work merges into
  `integration` (`main` by default); releases are tags on it, cut by the
  orchestrator (below). Bridle's own project sets nothing, since the default
  already matches.
- **Dev + release** (`release` set): work merges into `integration` (e.g.
  `dev`); `release` (e.g. `main`) only moves when `integration` is merged
  into it for a release — done by the orchestrator or the human, not by the
  ordinary worker/manager merge described below.

When `integration` is unset the daemon requires a local `main` branch and refuses to
start without one, naming `branches.integration` (a `master` project sets it explicitly).

Every role's system prompt states the project's actual `integration`/
`release` branches as a plain sentence (`bridle-daemon::config::
stable_system_prompt`), and `workflow/base/skills/worker/SKILL.md` and
the base role prompts (`workflow/base/roles/*.md`) and manager skill reference them as `{{branches.integration}}`
/ `{{branches.release}}`, the same templating `{{commands.check}}` already
uses (docs/design/workflow-layers.md, "Per-project command bindings") — so
no prompt hardcodes a branch name. The role prompts and skills likewise use
`{{commands.check}}` for the definition-of-done command.

**Rules reach daemon-spawned agents in the system prompt.** After the role prompt, every
spawned, resumed or renewed agent's system prompt carries `## Workflow rules`: its role's
resolved rules from the base, pack and project layers (`Config::role_rules_text`, called from
`supervisor.rs` on spawn, resume and renew; [[roles-and-config]]). Role prompts cite rules by
id (`kiss`, `yagni`), not by bridle's own paths. Component (L4) rules, facts and guides are not
included. The orchestrator and advisor sessions are not daemon-spawned: their opening prompt is
`bridle prime orchestrator|advisor`, which carries no resolved rules. `bridle sync`'s CLAUDE.md
block, which points at the rule files, only exists where someone ran `sync`; bridle's own
CLAUDE.md has none.

**Enforcement, not just prose.** When `release` is set, every role except
`orchestrator` gets `Bash(git push origin <release>)` added to its
`disallowed_tools` automatically (`config::apply_branches`) — additive and
so, per the override semantics in
[[docs/design/workflow-layers|workflow-layers.md]], **locked**: a project
config can't remove it. The orchestrator is exempt because cutting a release
is its job. Workers already can't push at all
(`Bash(git push *)` in their built-in `disallowed_tools`); this closes the
one door a manager would otherwise have to the release branch.

### Trial onboarding

Per `workflow/base/rules/existing-projects.md`: the human's real projects are
never touched by an agent without review. An onboarding is a **trial**: the
orchestrator clones the project's real integration branch (`main` or `dev`)
to a fixed branch, `bridle-adopt`, pushes it, and the trial's own
`.bridle/config.toml` sets `[branches] integration = "bridle-adopt"` (with
`release` left unset — a trial never cuts a release). `bridle-adopt` sits
outside the `bridle/<agent>` branch namespace (a `-`, not a `/`, after
`bridle`), so it can't collide with an agent's own branch; `validate_agent_name`
(`worktree.rs`) separately reserves `state`, since `bridle/state` is the
state branch. Nothing about the setting itself is trial-specific: it's the
same `integration` key, pointed at a different value, so `main`/`dev` are
mechanically never a push or merge target for the trial's own agents at all,
not just conventionally avoided.

**Where the manager merges, during a trial.** The manager's `workdir = repo`
role runs in the daemon's one main clone, which the human also uses
day-to-day for the project's real branches. Checking that clone out to
`bridle-adopt` would disturb whatever the human has checked out there. The
simpler alternative — and the one bridle uses — is to leave the main clone
alone and give the trial's manager a `workdir` pointing at a dedicated
worktree checked out on `bridle-adopt` instead (`bridle agent spawn manager
--cwd <trial-worktree>`; `Workdir::Path` already exists for exactly
this, `supervisor.rs`). No new mechanism: the trial just uses the spawn-time
override every role already has, instead of the role's own `workdir =
"repo"` default.

## Merging completed work

**Bridle merges its own completed work into the integration branch**; the
human doesn't have to. Progress would otherwise stop at every finished
branch. The manager does it, or the orchestrator when there's no manager, in
the clone:

1. The worker brings its branch up to date: it merges the **local**
   integration branch (`git merge --no-ff <integration>`, never `origin/*`)
   into `bridle/<agent>`, resolves any conflicts, runs `just check` (or the
   project's equivalent) and commits. A worker never touches the integration
   branch directly and never fetches or merges from a remote.
2. The merger checks that the integration branch is an ancestor of the
   branch (`git merge-base --is-ancestor <integration> bridle/<agent>`),
   that the worker's worktree is clean, and that the diff does what the task
   asked and nothing else. The merger verifies the check before landing, never on the worker's word alone.
   Anything short of that goes back to the worker.
3. `bridle land <task-id>` (the integrator, [[roles-and-config|roles and config]]): under a
   daemon-wide lock, it squash-merges the branch into one commit (subject `<task id>: <title>`, the task summary as
   body, `Task:` and `Branch:` trailers) in `<workspace>/integration`, runs `[integration] check` there (skipped on a fast-forward of an unchanged base), and only then moves the
   integration branch (if it moved meanwhile only by commits under `[integration] check_skip_paths`, e.g. `docs/**`, the squash is replayed on the new tip without re-checking; any other move fails "moved, retry"), so a red or conflicting merge never lands. The branch is found from the
   task's claimant, or `--branch`. The integration branch reads as one commit per task (sq4m, tr7k); the
   `Branch:` trailer is how the landed branch is recognised as merged, unless the branch has commits
   newer than the landing (a reused branch), which stay unmerged (x3xk).
4. `git push origin <integration>`, straight after the merge, so the remote
   never lags the clone. `land` never pushes: only the merger pushes, and only the integration
   branch and (trunk pattern) release tags; workers never push. The release
   branch, when a project has one, is never a target of this step at all
   (mechanically denied — see "Branch pattern", above).
   **Origin watch** (k6jd): the manager (`Bash(git *)`) and the orchestrator
   (`Bash(git fetch origin)`, read-only) may fetch; workers may not. Every 10 minutes the
   daemon runs `git fetch origin` in the owner's clone and compares the integration branch
   with `origin/<integration>` (`git rev-list --left-right --count`). Any difference is a
   `git.diverged` event (`{integration, ahead, behind}`; zeros when back in step) and one
   note to the orchestrator, sent again only when the counts change. `bridle daemon doctor`
   shows the same as "N ahead, M behind", and a failed fetch as a warning. Silent with no
   `origin` or no such branch there.
5. `land` then does what `bridle task done <id> --commit <sha> --branch bridle/<agent>` does
   (still available by hand): the daemon
   refuses unless `<sha>` is reachable from the integration branch, marks the task
   `integrated`, tells other workers `main moved`, then removes
   every agent on that branch, its worktree and the now-landed branch (`-D`), and notes what it
   removed on the task. Unmerged work can't be lost. `bridle rm --delete-branch` still works for
   one agent, and treats a `Branch: bridle/<agent>` trailer on a commit
   reachable from `HEAD` as landed. `bridle status` lists stopped agents whose
   branch has landed (`merged_leftovers`; an empty branch isn't landed, and `daemon_shutdown`
   agents are left out), as a safety net. Landed branches aren't kept; the
   landing commit on the integration branch is the record (the human, 2026-09-28).

The orchestrator verifies the integration branch after each merge (`just
check`, twice, off load). If it's red, nothing else merges until it's green
again; the fix goes forward as a normal task.

### Releases

Bridle is versioned with [SemVer](https://semver.org). The version lives in
`[workspace.package]` in `Cargo.toml`; the tag is `vX.Y.Z`, annotated, on a
`main` commit the orchestrator has verified green. Before 1.0:

- **minor** (`0.x.0`): a build-order phase or agent-host item is complete
  (for example P0);
- **patch** (`0.x.y`): fixes and small improvements since the last tag, when
  they're worth marking;
- docs-only changes don't get a release.

The orchestrator cuts releases: it bumps the version in one commit on `main`,
tags it, and pushes both. A missed or late tag costs nothing, so releases
wait for an orchestrator to be live. For a dev+release project, cutting a
release is also the orchestrator's (or the human's) job: merging
`integration` into `release` and pushing `release` is exactly the one case
"Branch pattern" above carves out of the mechanical deny — building that
release-cut flow itself is future work (out of scope for the branch-pattern
setting described here).

**Escalate to the human instead of merging** when the change is significant:
it rewrites a design decision rather than implementing one, changes what is
human-only or how tokens and containment work, migrates the store in a way
that drops data, deletes work, or the worker or merger flags it for review.
Send `bridle send human --question`, and keep other work moving.

## Several projects at once

Several projects run at the same time on different repos, **entirely
separately**: each has its own workspace, daemon, database and roles, and later
its own tasks, workflow layers and design tree. Nothing is shared between
daemons, so one project's rules can't leak into another's.

To make that workable from one terminal, TUI or orchestrator:

- **Registry.** On start, each daemon writes
  `~/.bridle/daemons/<project>.json` (`{project, workspace, repo, url, pid,
  started_at}`), and removes it on clean shutdown. Stale entries (dead pid) are
  pruned by any reader. `project` defaults to the clone's directory name and is
  set with `--project`. Two daemons can't claim the same name.
- **Selection.** `bridle --project <name> …` (or `$BRIDLE_PROJECT`) targets a
  daemon from the registry. `bridle daemon list` lists them. Inside a workspace,
  the cwd walk picks the right one without flags.
- **Ports.** Each daemon listens on `127.0.0.1:0` by default, so they never
  collide. The actual URL is in `daemon.json` and the registry.
- **Tokens are per daemon.** The human has one token per workspace, and an
  external orchestrator gets one per project it's allowed to drive.
- **A TUI or orchestrator that spans projects** fans out over the registry.
  There's no cross-project daemon.

**The one shared resource is the subscription budget.** All projects draw on
the same account windows ([[docs/design/usage-and-budget|usage and budget]]).
Each daemon records only its own usage:
[[how-project-daemons-share-one-budget-xypj|how project daemons share one budget]].

**Hosting the orchestrator in bridle.** If the human can't run their own agent,
they can host one (`bridle spawn orchestrator`, a role like any other) and talk
to it with `bridle send`. Where the one orchestrator should live is open:
[[where-the-single-orchestrator-lives-hj4g|where the orchestrator lives]].
