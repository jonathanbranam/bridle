# The CLI

Every command is a thin client of the daemon's API
([[docs/design/agent-host/api|API]]). Every command takes `--json`, which agents
always use; humans get compact tables.

## Built

```
bridle [--url URL] [--project NAME] [--token T] [--json] <command>

bridle serve   [--repo PATH] [--workspace DIR] [--listen ADDR] [--detach] [--take-over]
bridle stop-daemon                            prints "requested shutdown", "acknowledged; the daemon is stopping N agents,
                                              up to Ns" (the daemon's stop_grace + 5 s), "N agents still running" as the count drops,
                                              then "shutdown complete (Ns)"; after 60 s it errors, pointing at `bridle daemons`
                                              and <workspace>/.bridle/daemon.log
bridle restart [--wait SECS] [--upgrade]                restart the daemon in place once every agent is idle (orchestrator or human); prints the commit and the agents to
                                              resume, then "the daemon is back". A busy daemon (nothing idle within --wait, default 600) errors and stays up. --upgrade first builds the newest green-CI commit on main (background; prints "building <sha>" or "nothing to upgrade" and returns; the daemon restarts itself after the build)
bridle init    [--repo PATH] [--name N] [--integration BRANCH] [--stack S]  scaffold .bridle/config.toml + .gitignore; never overwrites
bridle doctor  [--repo PATH]                 check the project's setup, say what to fix; exit 1 on a failure
bridle launchd install [--repo PATH] [--workspace DIR] [--force]   macOS: write the LaunchAgent plist, print launchctl commands
bridle launchd uninstall                    remove the plist, print the bootout command
bridle rebuild [--from-origin]              first fetches origin/bridle/state (fast-forward only); reconstructs tasks/edges/open_questions/claims
                                              (claims.toml) from the state branch alone; the migration path for a fresh
                                              clone with no bridle.db yet; also restores the handover notes (handovers/<id>.md)
bridle daemons                              # every running project daemon on this machine, with agent counts
bridle status                               # daemon, agents, active incidents, Claude Code version, the last wake delivered and whether a waiter is open, last CI result (sha, conclusion, age, url) when [ci] github is on; state branch push (age, or the failure) when [state] push is on
bridle spawn   <role> [--name N] [--prompt TEXT | --prompt-file FILE]
               [--worktree [--base REF] | --in-repo | --cwd PATH] [--model M]
               [--allow-tool TOOL ...] [--env KEY=VALUE ...] [--ignore-budget]
               [--component ID ...]
bridle agents  [--all]
bridle show    <agent>
bridle send    <agent|human|role:NAME> [TEXT | --text-file FILE] [--question] [--when now|idle] [--reply-to ID] [--task ID]
bridle inbox   [--all] [--mark-read]        # messages to me, plus every task's open question (list)
bridle inbox show <id> [--mark-read]        # show one message in full; leaves it unread unless --mark-read
bridle inbox read <id>...                   # mark one or more messages read
bridle inbox unread <id>...                 # mark one or more messages unread again
bridle ask     <task-id> TEXT [--to WHO]         question against a task; blocks it until answered, and sends a pointer message (kind question) to WHO (agent, role:NAME, external:NAME, human); default: the caller's spawner, or human
bridle answer  <task-id> TEXT                    answers a task's open question; frees it to be ready again; sends the asker a pointer (kind answer)
bridle claim   <task-id>                         claims a ready task for the caller: planned -> claimed
bridle release <task-id>                         releases the caller's own claim: claimed -> planned
bridle ready   [--all] [--role R]                the highest queue tier with a startable task (planned, deps met, no open question, unclaimed)
bridle queue                                     read-only: claimed tasks with their worker, then the tiers in rank order
bridle queue set --tier T,T... [--tier T,T...]   replace the whole queue, one --tier per tier (PM or human only)
bridle queue add-tier <task>...                  append one tier at the back (PM or human only)
bridle dep add|rm <task> (--to OTHER [--kind K] | --blocked-by OTHER)   K: blocks (default)|parent|discovered-from|related|supersedes|duplicates
bridle wait    <task> [--until STATE] [--or-message] [--timeout SECS]   block until the task changes state; exit 4 on timeout
bridle interrupt <agent> [--drop-held]
bridle stop    <agent> [--now]      bridle resume <agent> [--ignore-budget]
bridle renew   <agent> [--ignore-budget]    stop + fresh process/session, same worktree/branch/role/model
bridle rm      <agent> [--force] [--delete-branch]
bridle logs    <agent> [--follow] [--raw] [--since LINE]
bridle events  [--follow] [--since SEQ] [--agent A] [--kind PREFIX]
bridle usage   [--by role|model|agent] [--since DURATION]   # DURATION: <n>s|m|h|d, e.g. 30d
bridle cost audit [--check]                 static: size of what bridle injects into agent context (usage-and-budget.md)
bridle tui                                  interactive terminal UI: agents list, live event tail
bridle budget [--schedule]                  usage governor status, or the whole resolved schedule (usage-and-budget.md)
bridle budget hold [--for D | --until HH:MM] | release          idle the account for the human / end the hold early
bridle budget override <period|default> [--until HH:MM] | override-clear   force a schedule period's thresholds / revert
bridle budget max-workers <N> | --clear     live worker cap, lost on daemon restart
bridle token create <name>                  with a known project (--project, or the cwd's daemon) the token is saved in
                                             ~/.bridle/credentials.toml and not printed; with --url it's printed once
bridle token list                           name, created-at, revoked-or-not; never the token itself
bridle token revoke <name>                  human only, external tokens only (an agent's own token is
                                             revoked through `bridle rm`, not this); also removes its
                                             credentials.toml entry for the project
bridle statusline                           Claude Code statusLine command; local only, no daemon call
bridle orchestrator note-session            the orchestrator launcher's SessionStart hook: writes $BRIDLE_HOME/orchestrator.session
                                             from the hook JSON on stdin; local only; never fails
bridle handover done                       the orchestrator's state is written: the daemon stops and relaunches its session (marker only); human and external:orchestrator only
bridle handover write --file <path>|-      record the orchestrator's handover note (human and external:orchestrator only); prints its id
bridle handover list | show <id>           the notes, newest first · one note
bridle wait-for-wake                        the orchestrator's background watcher: waits for a wake condition, prints it and exits 0 (`nothing` after 25 min); external:orchestrator only
bridle arch-guard                          Claude Code PreToolUse hook: blocks design/architecture/ edits outside an arch-revision task
bridle stop-check                           Claude Code Stop hook for the worker role; refuses to stop
                                             with an unreleased claim and no thread entry since claiming
                                             it (docs/design/coordination.md); never fails
bridle rules explain <id>                   which layer wins a rule id, and what it shadowed
bridle rules diff --project-layer           everything the project layer does differently from
                                             the base/pack layers below it
bridle rules explain|diff ... --component <id>  the same on top of that component's chain (L4,
                                             root-most ancestor first); diff shows what each
                                             component layer changes instead of the project layer
bridle sync                                 renders resolved workflow layers into CLAUDE.md's
                                             managed block, .claude/skills, .claude/agents and
                                             .claude/settings.json's hooks; local only, no daemon call
bridle arch list [--invariants] [--root DIR]   lists architecture elements (id, `invariant` flag, title,
                                             file:line) from `*.md` under DIR (default
                                             `design/architecture`); a missing or duplicate `a-` id
                                             is an error (diagnostics on stderr, exit 1); --json
                                             prints the elements with their text; local only
bridle arch propose --title T (--argument TEXT | --argument-file FILE|-) [--arch-root DIR]   creates an `arch-revision` task with the proposal
                                             (daemon call); validates that the architecture
                                             directory exists locally
bridle trace down|up <id>  [--goals DIR] [--arch DIR] [--specs DIR]   walks the trace links across
                                             `design/goals`, `design/architecture` and `design/specs`:
                                             `down` lists everything depending on the element, `up` what it
                                             rests on, up to the goals (indented by distance; --json gives
                                             rows with `depth`); an unknown id, an unknown link target or
                                             any parse error is an error (exit 1); local only
bridle trace orphans                         requirements with no `traces=` (a warning on stderr, exit 0)
bridle trace suspect                         links whose recorded `@hash` differs from the upstream element's
                                             current hash (id, file:line, upstream, recorded, current; --json
                                             prints them); exit 1 if any
bridle trace confirm <id>                    rewrites that requirement's link hashes to the current ones; edits
                                             only the heading line, the rest of the file byte-for-byte
bridle explore check [paths...]              checks exploration findings frontmatter (default `design/explore`);
                                             diagnostics on stdout, exit 1 on any error; local only
bridle explore new|conclude|abandon <id>     scaffolds `design/explore/<id>/findings.md` (status open;
                                             refuses to overwrite) or rewrites just its `status:` line
bridle pane tag <name>                       set the @bridle tmux pane option to the given name (errors
                                             if not in a tmux pane); used by launcher scripts
bridle pane untag                            clear the @bridle tmux pane option (errors if not in a
                                             tmux pane)
bridle spec check [paths...] [--root DIR] [--require-ids]   validates spec files (dirs are searched for
                                             *.md; default `design/specs`, or --root) with the
                                             bridle-spec parser: prints file:line:col: message per
                                             diagnostic and a summary, exit 1 on any error; a
                                             requirement without an id is a warning (an error with
                                             --require-ids); --json prints them as structured output;
                                             local only, no daemon call
bridle spec id [paths...] [--root DIR] [--ledger FILE] [--dry-run]   writes a stable id (`{#r-xxxx}` /
                                             `{#s-xxxx}`) into every requirement and scenario heading
                                             lacking one, in place, touching only those heading lines;
                                             ids are unique across the files processed and never reused
                                             (ledger `<root>/.ids`); idempotent; `--dry-run` prints
                                             the plan and writes nothing; local only, no daemon call
bridle spec export --format gherkin|json [--out DIR] [paths...] [--root DIR] [--scenario ID]... [--task ID]
                                             exports specs for test runners (same path defaults as
                                             `spec check`); gherkin: one <capability>.feature per spec
                                             (Rule per requirement, executable scenarios only, tagged
                                             with their @tags and id, e.g. @s-b310) into --out,
                                             default `.bridle/cache/features/` (gitignored); json: the
                                             whole AST (documented in specs-to-tests.md) on stdout, or
                                             `specs.json` in --out; refuses, printing the diagnostics,
                                             when any spec has errors; `--scenario s-xxxx|r-xxxx`
                                             (repeatable) keeps only those scenarios (an `r-` id keeps
                                             all its scenarios), dropping requirements and specs with
                                             none selected; `--task ID` selects the scenarios in the
                                             task's declared impact (modify, add-under, remove ids;
                                             exits 1 if it declares none) and asks the daemon, else
                                             local only
bridle spec coverage [--root DIR] [--tests DIR ...] [--require-all] [--json]
                                             lists executable scenarios whose id does not appear in
                                             test sources; scans text files (skip binary, node_modules,
                                             target, .git) under --tests directories (default `tests`
                                             and `test` if present) for the token 's-' plus hex ids
                                             of scenarios; outputs counts (executable, bound, unbound)
                                             and the unbound list as 'file:line: s-id title'; exits 1
                                             with --require-all if any unbound (default exit 0); local
                                             only, no daemon call
bridle goals list [--root DIR] [--priority P] [--stance S]   lists goals (docs/design/goals-tier.md) from
                                             `*.md` under --root (default `design/goals`): id, firmness,
                                             priority, stance, title per line; the stance is defaulted from
                                             the priority when not written; --json prints the goals and
                                             diagnostics; diagnostics go to stderr as file:line:col: message
                                             (an unaddressed goal without a why is a warning); exit 1 on
                                             any error; local only, no daemon call
bridle goals propose <goal-id> --change KEY=VALUE --why TEXT [--goals-root DIR]  creates a task proposing a change to the goal's
                                             firmness, priority, or stance (repeatable --change;
                                             daemon call); validates that the goal exists
bridle spec import openspec [--from DIR] [--to DIR] [--dry-run]   moves each `<from>/<cap>/spec.md`
                                             (default `openspec/specs`) to `<to>/<cap>.md` (default
                                             `design/specs`) with `git mv` (plain rename if untracked),
                                             then assigns ids as `spec id` does; parses first and
                                             changes nothing on any error or existing target;
                                             `.feature` files, changes, config left in place;
                                             idempotent; local only, no daemon call
bridle prime orchestrator                   fresh session's opening context: role prompt, current
                                             state, startup steps; local only, no daemon call
bridle prime worker|planner [--component ID ...] [--task ID]   the role's rules, facts, guides, plus named components' scope; --task is worker only
bridle task new    <title> -k/--kind KIND [--body TEXT | --body-file FILE] [--component ID ...] [--size S|M|L] [--for-human]
bridle task show   <id>
bridle task plan   <id>                                                 open -> planned: ready to build, claimable once unblocked
bridle task edit   <id> [--title TEXT] [--body TEXT | --body-file FILE] [--component ID ... | --no-component] [--size S|M|L|none]
bridle task list   [--claimed-by WHO] [--component ID] [-k KIND]             WHO: me|human|<agent name>|<principal id>; unclaimed tasks have no claimant to match
bridle task search <words...>                                      search for tasks by words in title/body/summary (case-insensitive substring match, all words must match); includes done and dropped tasks
bridle task drop   <id> --reason TEXT
                                                                        incidents are `-k incident` tasks (`task list -k incident` lists them); plan/done/drop/reopen of one is orchestrator/human only, and `plan` sends the notice to every agent; see agent-host/incidents.md
bridle task done   <id> [--commit SHA] [--branch NAME] [--resolution TEXT]  -> integrated; `--commit` is required unless the human claimed the task (a to-do) or the task is an incident (`--resolution` goes in its thread and the "resolved" note); records the sha (and branch) on the task and in the thread; with --branch removes the branch's agents, worktree and branch; warns if no summary
bridle impact set  <task> [--modify ID].. [--add-under ID].. [--remove ID].. [--files GLOB..]  declares the task's impact, replacing any earlier one; only an open/planned/claimed task; ids checked by shape (r-/s-/g-/a- + hex) only
bridle impact check [--specs DIR]                                overlaps between in-flight tasks' declared impact, plus merge probes of claimed tasks' branches (`--json`: `{overlaps:[{level,tasks,kind,key}]}`); exit 1 if any is a conflict; see impact-and-conflicts.md
bridle land <task> [--branch B] [--check-cmd CMD]              the integrator: squash the branch into one commit (`<task id>: <title>`, the summary as body, `Task:`/`Branch:` trailers) in the integration worktree, run `[integration] check` (skipped, with a note, when the integration branch is an ancestor of the branch tip), fail if the nextest test count is outside the sane band around the last full run (`<workspace>/last-full-test-count`), fast-forward the integration branch (guarded), mark the task done; any failure lands nothing (exit 1); never pushes
bridle probe <task-or-agent> | --branch B                       `git merge-tree` of the branch into the integration branch, no working-tree change (`--json`: `{branch,against,outcome,paths}`); exit 1 on a conflict; needs git 2.38
bridle port alloc [--pid N] [--label L]                         allocate a free port from `[ports] range` (not reserved, allocated or listening); prints it (`--json`: the allocation); recorded against you and your claimed task
bridle port release <port>                                      free a port
bridle port list                                                allocated ports (`--json`: array of `{port,agent,task,pid,label,allocated_at}`)
bridle conflict list                                            conflicts opened by `impact check`, open first (`--json`: array of `{id,tasks,kind,key,state,resolution,opened_at,resolved_at}`)
bridle conflict resolve <C12> --compatible <reason> | --order A,B | --merge-into <task>   record the outcome (exactly one flag); `--order` adds an `A blocks B` edge; A, B and the task must be the conflict's two tasks; a resolved conflict can't be resolved again
bridle impact show <task>                                         prints the declared impact (`--json`: the impact object)
bridle task summary <id> --text TEXT | --file FILE                    records how it was implemented; `-` reads stdin; replaces an earlier summary
bridle task reopen <id>
bridle task note   <id> [TEXT | --text-file FILE] [--notify AGENT]  plain note to the task's thread; no effect on readiness
```

- **`--allow-tool TOOL`** on `spawn` (repeatable) grants a tool beyond the
  role's `allowed_tools` for this one spawn only — no config or role change,
  and it never touches `disallowed_tools`
  ([[docs/design/agent-host/roles-and-config#Per-spawn tool overrides|roles-and-config.md]]).
- **`--env KEY=VALUE`** on `spawn` (repeatable) sets an environment variable
  in this one spawn's process only, e.g. a secret — no config or role
  change, not carried by a later spawn or `resume`, and never logged or
  returned by read endpoints
  ([[docs/design/agent-host/roles-and-config#Per-spawn secrets|roles-and-config.md]]).
- **`--ignore-budget`** on `spawn`/`resume` skips the budget governor's
  holding/paused refusal for that one call (`renew` never waits on a hold: it
  replaces a session rather than adding load, so the flag is accepted and ignored)
  ([[docs/design/usage-and-budget#Resuming|the escape hatch]]).
- **`budget hold`/`release`** apply to the current daemon only; see
  [[docs/design/usage-and-budget#The human's hold|the human's hold]] for the
  cross-daemon gap.

- **`budget`** prints the applied `five_hour` thresholds with their source
  (override / schedule period / default), the current period's span, the next
  schedule change, a `why` line per non-normal window, and each window's
  status (when not `allowed`) and reading age, marked `(stale)` past
  `max_staleness`. `--schedule` prints the whole resolved schedule instead.
  Text times are local machine time; `--json` keeps UTC/ISO 8601
  ([[docs/design/usage-and-budget#Seeing what applies|details]]).

- **`send`**: an empty or whitespace-only text is refused (by the CLI, and by the daemon
  with 400), here and in `task note`. When given `--text-file FILE` or `--prompt-file FILE`, pass `-`
  as the filename to read from stdin instead. This avoids passing backticks and
  other shell metacharacters as command-line arguments, which can trigger
  permission denials in Claude Code. Example: `echo "message" | bridle send w1 --text-file -`.
  `role:NAME` fans the message out to every live agent currently holding that
  role — one delivered message per matching agent, same as sending to each
  individually; `bridle send` prints one `sent <id> -> <to>` line per recipient.
  A role with no live agents is an error, same as an unknown agent name.
  `--task <id>` (and `bridle task note <id> --notify <agent>`, the same call) writes the text as a note on the task's thread and sends the recipient `<id>: note added` plus its first line; an unknown task is an error and nothing is sent.

- **`task new/edit/note`**: when given `--body-file FILE` or `--text-file FILE`, pass `-`
  as the filename to read from stdin instead. This avoids passing backticks and
  other shell metacharacters as command-line arguments, which can trigger
  permission denials in Claude Code. Example: `cat long-body.txt | bridle task new "title" -k feature --body-file -`.
  `--body-file` is mutually exclusive with `--body`; `--text-file` is mutually exclusive with the positional `TEXT` argument.

- **`rebuild`** is `TaskManager::rebuild_from_state_branch` (docs/design/storage.md,
  "Rebuild"): the migration path for a fresh clone with no `bridle.db` — clone the repo,
  start the daemon, `bridle rebuild`. `--from-origin` first fetches `origin/bridle/state`
  (`POST /v1/rebuild?from_origin=true`; fast-forward only, never overwrites a local branch with
  state of its own, and says what it did). Human-only; refuses (409) rather than overwrites if
  the database already has any tasks, edges or open questions. Claims are never
  reconstructed — they're SQLite-only, with no state-branch counterpart, so any in-flight
  claim is simply lost, which is correct here, not a gap.
- **Landing record.** `task done --commit SHA [--branch NAME]` stores `commit` and `branch` on
  the task; `task summary <id>` stores a short implementation summary (any state; a second call
  replaces it). `task show` prints branch, commit and summary together, so a task id leads to
  `git show --stat <commit>`. `task done` warns on stderr, but succeeds, when no summary exists.
  With `--branch`, `task done` also cleans up in the daemon: it refuses (409) unless `--commit` is
  reachable from the integration branch, then removes every agent on that branch (stopping running
  ones, forced), their worktree and the branch (`-D`; a squashed branch isn't an ancestor), and
  notes what it removed on the task's thread. A failed removal is noted, not fatal. `bridle status`
  lists stopped agents whose branch has already landed (at least one commit of its own reached
  `HEAD`; an empty branch is no work) so leftovers show up, leaving out agents stopped by a daemon
  shutdown, which are due a resume.
- **Human to-dos.** `task new --for-human` creates the task `planned` and claimed by the
  `human` principal in one step and sends the human one inbox message naming it. The human
  finishes it with `task done <id>` (no `--commit`); `task list --claimed-by human` lists
  the open ones. By convention the title starts `[at restart]` or `[at next reboot]` when it
  must wait for one; nothing parses it. See coordination.md, "Human to-dos".
- **`--size S|M|L|none`** on `task new`/`task edit` sets the task's optional estimated size
  (case-insensitive), so small tasks can be picked when budget runs short. `--size none`
  on `task edit` clears the task's size. It's informational: nothing selects on it and the
  queue's order stays PM-owned. `task show` prints it when set; `task list`, `queue` and
  `ready` rows have a SIZE column (`-` when unset).
- **`--component ID`** (docs/design/components.md) scopes a task or spawn to a
  `[components.<id>]`. `task new`/`task edit`/`spawn` reject an unknown id (the
  daemon checks it against its config); repeats collapse. `task edit --component`
  replaces the list, `--no-component` clears it (repo-wide). `task new` with none
  given prints a one-line reminder on stderr, only in a project that defines
  components (read from `.bridle/config.toml` in the cwd), and never refuses.
  `task list --component X` matches tasks naming `X` or any descendant of it
  (naming a child implies its ancestors), and combines with `--claimed-by`.
  `task show` prints the list. `spawn` without `--component` defaults to the list
  of a task the spawner has claimed, else none. The daemon stores the list on the
  agent (`agents.components`, shown by `show`) and sets `BRIDLE_COMPONENTS`
  (comma-separated) in its process env, also on `resume`/`renew`.
- **`task list --claimed-by`** filters on the task's current claimant
  (`Task::claimed_by`, docs/design/storage.md). `me` resolves to the calling
  principal's own id, the same as `--to me` on `send`/`inbox`; anything else
  is looked up as an agent name first, then matched against `claimed_by`
  verbatim, so a full principal id (`agent:w1`, `human`) works too. A task
  with no claim never matches any filter value.
- **Discovery** of the daemon, and **which token** the CLI uses (including `$BRIDLE_AS` and `~/.bridle/credentials.toml`), are in
  [[docs/design/agent-host/daemon#Workspace layout|workspace layout]] and
  [[docs/design/agent-host/principals#How the CLI picks a token|principals]].
  `--project` also reads `$BRIDLE_PROJECT`, including for `serve`, where it
  names the project being served.
- **Exit codes**: 0 ok, 1 error, 2 usage error, 3 daemon unreachable (every
  discovery failure, including an unknown `--project`), 4 `wait` timed out.
- **`wait <task> [--until <state>] [--or-message] [--timeout <secs>]`** blocks on the SSE
  event stream (no polling) until the task's next state change, or until it is in
  `--until` (returning at once if it already is). `--or-message` also returns on a message to
  the caller, including one already unread. `--timeout` exits 4. Prints one line
  (`<task> is <state>` / `message <id> arrived`); `--json` prints `{result: state|message|timeout,
  task, state, message}`. Meant to run as a background Bash so the caller is woken.
- **`logs`** renders the transcript's output lines; `--raw` prints every line
  verbatim. Without `--since` it shows the latest lines (tail), not the
  oldest; give `--since` to page forward from a line number instead.
  `--follow` polls once a second.
- **`events`** without `--follow` returns the most recent 500 matching
  events, oldest first; give `--since` to page forward from a cursor instead.
  `--follow` streams over SSE, filtering agent and kind on the client, and
  starts at the tail unless given `--since` (or resuming after a
  reconnect), in which case it backfills from that cursor first.
- **`daemons`** hits every registered daemon's unauthenticated `GET /v1/health`
  concurrently, with a ~1 s timeout each, to show non-terminal agent counts
  without a cross-daemon token. A daemon that doesn't answer in time (dead,
  slow, unreachable) shows `?` (`null` under `--json`) instead of blocking on
  it.
- **`cost audit`** is local and static: no daemon call, just `.bridle/config.toml` and
  `.bridle/cost-baseline.json` read from the current directory. It measures each role's
  rendered system-prompt file (the part meant to be identical across agents of a role,
  `stable_system_prompt`) from a fresh render, and reports it next to the committed
  baseline. The other categories the design names (prime, skill descriptions, MCP tool
  schemas, hook boilerplate) aren't measured yet because they don't exist in bridle
  today; see `bridle_daemon::cost_audit`. Without `--check` it only reports; with it,
  exit 1 if any role grew more than
  `bridle_daemon::cost_audit::GROWTH_THRESHOLD_PERCENT` over baseline.
- **`rules explain`/`rules diff`** are local and static, like `cost audit`: no daemon
  call, just `.bridle/config.toml` and the layer directories it points at, read from
  the current directory ([[docs/design/workflow-layers|workflow layers]],
  `bridle_daemon::rules`). They resolve L1 base, L2 packs and L3 project rule layers
  by id, later layers winning unless an earlier one marks the id `locked: true`; a
  layer that redefines an id must give an `override` kind (`replace`, `append` or
  `disable`) — silent redefinition is an error, and `disable` requires a `reason`.
  L0 core has no file-backed layer yet (nothing in the binary defines rules that way
  today). `--component <id>` appends that component's chain (`[components.*]` in config,
  rules in `.bridle/components/<id>/rules`, root-most ancestor first) after L3, resolved
  on its own ([[docs/design/components|components]]); an unknown id is an error, and
  without the flag output is unchanged. `diff --component <id>` prints what each
  component layer defines, replaces, appends to or disables instead of the project layer.
  `explain <id>` prints which layer won and the full history of what it shadowed, in
  layer order; `diff --project-layer` prints every rule id the project layer
  (`<repo>/.bridle/rules`) defines, replaces, appends to or disables, relative to the
  base/pack layers below it — the flag is `--project-layer`, not `--project` as
  workflow-layers.md's own example reads, because `--project` is already the global
  flag that selects a daemon by project name and clap can't have both share that name
  with different value types. The L1 base and L2 pack layers come from `[rules]
  workflow = "path"` (relative paths resolve against the repo root) and `packs =
  ["name", ...]` in `.bridle/config.toml`; with no `workflow` set, or a `workflow`/pack
  directory that doesn't exist on disk, resolution just sees the project layer, not an
  error — `bridle-workflow`'s real location is still provisional (docs/questions/open/
  where-bridle-workflow-lives-r2uq.md), so nothing is guessed here. Pack layers are
  mechanism only for now: reading multiple `<workflow>/packs/<name>/rules` directories
  in listed order, with no real pack content yet (out of scope per workflow-layers.md).
- **`sync`** is local and static too, like `rules explain`/`diff`: no daemon call, just
  the layers `bridle_daemon::rules::discover_layers` finds, rendered by
  `bridle_daemon::sync` (docs/design/workflow-layers.md, "Rendering into what the agent
  harness reads"). `CLAUDE.md` gets a small managed block between
  `<!-- bridle:managed:start -->`/`<!-- bridle:managed:end -->` markers, inserted if
  absent and replaced in place otherwise — everything else in the file is untouched
  byte-for-byte. `.claude/skills/bridle-<name>/` and `.claude/agents/<role>.md` are
  fully regenerated every run from each layer's `skills/<name>/` and `agents/<role>.md`
  sources (not committed — gitignore them); a skill's `SKILL.md` is *appended* across
  layers (a project's own `skills/<name>/SKILL.md` reads as an addendum), while every
  other skill file and the whole of an agent file is replaced wholesale by the last
  layer that defines it. `.claude/settings.json`'s `hooks` object is merged from each
  layer's `hooks/<event>.json` (a JSON array of Claude Code hook-config entries for
  that event name); a gitignored sidecar, `.claude/.bridle-sync-hooks.json`, records
  exactly which entries the last sync wrote, so re-syncing (or a layer's hooks
  changing) only ever touches those, never a hook a human added by hand — a
  pre-existing hook entry sync didn't write is always left alone. Renders no L4
  component rules (delivered by `bridle prime` instead, docs/design/components.md), same as `rules explain`/`diff` above; wiring a
  `SessionStart` hook to run `sync` automatically is a follow-up, not built yet — for
  now it's a command you run yourself. `hooks/<event>.json`, and the "later layer wins
  wholesale" convention it and `agents/<role>.md` use, are this command's own
  convention; nothing in `workflow/` uses either yet. Skill sources may reference
  `{{commands.check}}`, substituted with `.bridle/config.toml`'s `[commands] check`
  (default `"just check"`, per-project — e.g. `"make check"`) so a base skill like
  `workflow/base/skills/worker/SKILL.md` doesn't hardcode one project's build tool.
- **`doctor [--repo PATH]`**: local checks on the clone (default: the current directory),
  each printed `ok`/`warn`/`FAIL` with a one-line fix (`--json`: the list). Git repo; the
  integration branch exists (the g3ck failure mode); `.bridle/config.toml` loads (the
  config loader's own error text); files it references exist (role `system_prompt`, `workflow`,
  packs, component `docs`); every role has a prompt (warn); `.gitignore` covers
  `.bridle/cache/`, `bridle.db` and `daemon.json` (warn); `bridle/state` exists once a
  `bridle.db` does (warn); `[ports]` range sane; git >= 2.38; `claude` on PATH; `gh` on PATH
  when `[ci] github` is on. Exits 1 if any check fails. It never fixes anything and does not
  talk to a daemon; a dry `sync` check isn't done because `sync` has no check mode.
- **`init [--repo PATH] [--name N] [--integration BRANCH] [--stack python|typescript|rust]`**:
  scaffolds a project, only what's absent and never overwriting (existing files are listed as
  skipped). `.bridle/config.toml` gets `[branches] integration` (the branch HEAD is on, or
  `--integration`), `workflow` (active only if the repo has `workflow/base/`, else a commented
  stub), `packs = [STACK]` with `--stack`, empty `manager`/`worker` roles (prompts come from
  `workflow` by default), and `[commands]`/`[integration] check` (`just check`, `cargo test`,
  `npm test` or `pytest`, by which of `justfile`/`Cargo.toml`/`package.json`/`pyproject.toml`
  exists; a commented stub if none) plus commented `[worktrees]` `setup`/`copy` stubs.
  `--name` is only a comment: config has no name key. `.gitignore` gets `.bridle/*.db*`,
  `.bridle/daemon.json` and `.bridle/cache/` appended if missing. Prints the next steps (`sync`,
  `doctor`, `serve`) and suggests `--stack` from the files it found; it runs none of them and
  asks nothing.
- **`launchd install|uninstall`** (macOS only): writes/removes
  `~/Library/LaunchAgents/dev.bridle.<project>.plist` (project from `--project`, else the repo
  directory name) and prints, but never runs, the `launchctl bootstrap`/`bootout` commands.
  The plist runs the absolute path of the current `bridle` as `--project <name> serve --repo
  <repo> --workspace <workspace>` with `WorkingDirectory` the clone, `RunAtLoad`, `KeepAlive`
  only on a non-zero exit (a deliberate `stop-daemon` stays stopped), output to
  `<workspace>/.bridle/daemon.log`, and `PATH`/`HOME` copied from the caller so `claude` and
  `git` are found. `install` refuses to overwrite without `--force`. Why: a daemon started by
  launchd has no GUI responsible app, so builds under it don't flash Gatekeeper's Verifying
  window (ticket qr8z). Moving a running daemon: `docs/context/launchd-restart-plan.md`.
- **`serve --detach`**: [[docs/design/agent-host/daemon#Running it|running the daemon]].
- **`tui`** is a subcommand, not a separate binary, so it shares `bridle`'s discovery,
  token and `--url`/`--project` flags like every other command. It's a thin client of
  `bridle-api`'s `Client`, with four views: an agents list (seeded from `GET
  /v1/agents`, kept live by `agent.state`/`agent.removed` events), a scrolling event
  tail (`events_stream`, which already reconnects on its own — see
  `crates/bridle-api/src/client/mod.rs`), the selected agent's transcript tail (polled
  from `Client::transcript` once a second, same model as `bridle logs --follow`), and
  an inbox of unread messages addressed to `me` (polled from `Client::list_messages`
  once a second, same query as `bridle inbox`). `Tab` switches between the four views,
  `j`/`k`/arrow keys scroll the focused one, `q`/`Esc` quits. The inbox view lists unread
  messages then every task's open question (as `bridle inbox` does), a question staying
  until it's answered. `Enter` opens the selected row in full (header and body as
  `bridle inbox show` prints them; opening doesn't mark it read); `q`/`Esc`/`Enter` closes
  it. `d`, on the list or in the opened message, marks a message done (read) without
  replying; questions are answered through the task, so `d` and `r` skip them. `r`, on the list or in the opened message,
  starts composing a reply to the message (simple line editing: insert,
  backspace, left/right, `Enter` to send, `Esc` to cancel); a submitted reply goes out
  via `Client::send` with `reply_to` set and `when: now`, then the original is marked
  read via `Client::mark_read` so it drops out of the unread list. Lives in its own
  crate, `crates/bridle-tui`, split Elm-style: a plain state struct and update function
  with no terminal/ratatui dependency (so it's unit-tested without a terminal),
  rendered by a separate `ui` module.
- **`task`** covers the `open`/`planned`/`claimed`/`dropped`/`integrated`/`reopened` states
  (docs/design/roles-and-lifecycle.md, Task lifecycle): create, show, edit (title/body,
  never state), list (id/title/kind/state), plan (`open` -> `planned`), drop (a reason is
  required, recorded in the task's thread), done (`--commit` required, recorded in the
  thread; the task becomes `integrated`, which resolves its `blocks` edges and drops it from
  `queue` and `ready`) and reopen (only a dropped or integrated task can be reopened).
  `in_review` and `accepted`, and everything that depends on those, arrive with later
  tasks — see the `Planned` block below.
- **`dep add|rm`** creates or removes one coordination edge (docs/design/coordination.md).
  `bridle dep add <task> --to <other> --kind <kind>` draws `<task> --kind--> <other>`;
  `bridle dep add <task> --blocked-by <other>` is sugar for `--kind blocks` with `from`
  and `to` swapped (`<task>` is blocked by `<other>`, so the edge runs the other way) and
  can't be combined with `--to`/`--kind`. `dep rm` takes the same shape. Edges can't
  connect a task to itself, and a repeat of the same `(from, to, kind)` triple is a
  conflict, not a silent no-op.
- **`inbox`** has three forms:
  - Bare `bridle inbox [--all] [--mark-read]` (list, the default) shows every message to
    `me` plus every task's open question: reads `GET /v1/messages` (with `to=me`,
    `unread=true` by default) and `GET /v1/questions`. `--all` drops the `unread` filter
    (shows read messages too); `--mark-read` calls `POST /v1/messages/{id}/read` on each
    message after listing, marking every one read. In JSON mode, returns both messages and
    questions; plain text prints a compact line per message/question.
  - `bridle inbox show <id> [--mark-read]` (show one message in full) fetches a single
    message to `me` by id (a question a delegate answered shows `Answered by:`; the list shows
    "answered by <who>: <first line>", visible with `--all`), prints the full header (from, kind, time, reply-to), the body,
    and the reply command (formatted as `bridle send <from> --reply-to <id> "..."`). It
    leaves the message unread (reading isn't handling); `--mark-read` calls
    `POST /v1/messages/{id}/read` after showing it. In JSON mode, returns the message object; plain text returns the
    formatted rendering above. Fails with a 404-like error if the message doesn't exist
    or isn't addressed to `me`.
  - `bridle inbox unread <id>...` calls `POST /v1/messages/{id}/unread` for each id, returning
    a read message to the unread list (JSON: `{"marked_unread": [...]}`).
  - `bridle inbox read <id>... ` (mark read, one or more) calls `POST /v1/messages/{id}/read`
    for each id in turn (the same endpoint `--mark-read` on list uses per message). Accepts
    multiple ids; useful for marking specific messages read without listing/marking
    everything else. In JSON mode, returns the list of ids that were marked; plain text
    prints a line per id.
- **`ask`/`answer`** are thin clients of `TaskManager::ask_question`/`answer_question`
  (docs/design/coordination.md, "Questions do not stop work"): `ask` appends a `question`
  thread entry and blocks the task's readiness until answered (`Conflict` if one is
  already open); `answer` appends an `answer` entry and clears the block. Neither takes a
  recipient — a question addressed to a task has no single recipient, per
  coordination.md's message table — so there's no `--to`; send-to-task is a
  later task.
- **`claim`/`release`** are thin clients of `TaskManager::claim_task`/`release_task`
  (docs/design/storage.md, "claims and leases"): `claim` moves a `planned`, unblocked
  task to `claimed` for the calling principal (`Conflict` if it isn't ready to claim —
  not planned, blocked, or already claimed); `release` moves it back to `planned`
  (`Conflict` if the caller isn't the current claimant, including if it isn't claimed at
  all). Neither takes a body: the claimant is always the caller's own token. A claimed
  task drops out of `ready` immediately, since `is_ready` requires `planned`; releasing
  it (explicitly, or via the lease-expiry tick) puts it back.
- **`ready [--all] [--role]`** lists the ready tasks of the highest queue tier that has a
  startable one: `planned`, unclaimed, no open question, no open `blocks` edge naming an
  unresolved blocker (roles-and-lifecycle.md, "ready is computed" and "the queue"; see
  coordination.md for exactly what "unresolved" means in this build). A task in no tier
  is backlog and never shown, even if it is ready. `--all` fans out
  across every daemon in the registry (`bridle daemons`), each with its own
  discovery-resolved token, instead of just the one daemon `--url`/`--project`/cwd
  discovery would pick. `--role` is accepted but a no-op: tasks don't carry a role field
  yet (a gap, not a design decision).
- **`queue`** without a subcommand is the read-only view; `set` and `add-tier` are PM-or-human
  only and stored in the state branch's `queue.toml` (roles-and-lifecycle.md, "the queue").
  `queue set` takes one `--tier a,b` per tier, in rank order, and replaces the whole queue.
- **`budget`'s** subcommands (`hold`, `release`, `override`, `override-clear`, `max-workers`)
  are described in [[docs/design/usage-and-budget|usage and budget]]. `max-workers` sets a live
  cap that never stops running workers, only blocks new spawns and resumes.
- **`orchestrator note-session`** is the SessionStart hook `scripts/claude-orchestrator` registers for
  its own session ([[orchestrator-supervision]]). It reads the hook JSON on stdin and writes
  `$BRIDLE_HOME/orchestrator.session` as `<session id> <transcript path>` (`/clear` gives the same
  process a new id). Local, silent, never fails.
- **`handover`** keeps the orchestrator's note as a record ([[orchestrator-supervision]] section 7):
  `write` reads a file or stdin (`-`), `list` shows id, time, author and first line, `show` the
  whole note. Latest wins; `bridle prime orchestrator` prints the newest under a heading with
  its age, and falls back to `docs/context/orchestrator-state.md` when there is no note (or no
  daemon to ask). **`handover done`** is `POST /v1/orchestrator/handover`
  ([[orchestrator-supervision]] section 6): the orchestrator runs it after writing its state, and
  the daemon stops the session (SIGTERM, SIGKILL after 15 s) and relaunches it at once. It
  stores nothing itself.
- **`wait-for-wake`** is `GET /v1/orchestrator/wake` ([[orchestrator-supervision]] section 5),
  replacing `scripts/orchestrator-watch.sh`. It prints each wake as `<reason>: <text>` and its
  detail as JSON (`--json`: the list of wakes), exits 0, and prints `nothing` when the daemon's
  25 minutes pass quietly. The orchestrator runs it in the background and starts it again on
  every exit. Any other principal gets a 403.
- **`events --kind orchestrator.context`** queries the orchestrator's context tracking (ct8m
  step 6, br-1fdb): emitted on the first reading of a session's tokens, on a lower reading
  (when compacted), and at most once per 10 minutes when the tokens change. Each event carries
  the session id, token count, context window size and uptime in seconds; use the query to
  answer "how long can the orchestrator run" with data points across sessions.
- **`statusline`** is Claude Code's `statusLine` command, configured in `settings.json`. It
  reads Claude Code's JSON on stdin and prints a short line back: model, context %
  (`context_window.used_percentage`) and token count (e.g., `40.0k`, `1.2M`) when available,
  the `5h`/`7d` rate-limit windows, the current folder and git branch, and the estimated
  session cost last, parenthesized. It's purely local — no daemon call, no token, never
  fails or hangs — since it runs on every render of the prompt. It does **not** call
  `POST /v1/statusline` any more (dropped in s8kn: the context governor gets account-wide
  windows from `get_usage` instead); that route and the `interactive_usage` table still
  exist in the daemon, unused for now, in case something needs per-invocation interactive
  snapshots later ([[docs/design/usage-and-budget#Where bridle can see usage|usage and budget]]).

  If `~/.bridle/statusline.token` holds a token, it also appends a short "N working · M for
  you" from `GET /v1/status` (agents in `working`/`starting`, and `unread_human_messages`) —
  a 2s-timeout, best-effort call: no token file, no daemon found, a timeout or an HTTP error
  all just skip the counts silently (logged at `tracing::debug`), never delaying or blanking
  the rest of the line. This is the one case where `statusline` does call the daemon, but
  never with `$BRIDLE_TOKEN` or the workspace's human token file — only this dedicated,
  per-user file (see [[statusline-bridle-counts-with-a-read-only-token-r7cs|r7cs]]).
  Deliberately not `$BRIDLE_TOKEN`: `resolve_token` checks it first, unconditionally, for
  every command, so exporting it in the shell profile would make every human-run bridle
  command (`stop-daemon`, `budget hold`/`override`, `token create`, ...) act as this token's
  principal instead of the human's. The token is **not** scoped read-only or to this route —
  bridle has no per-route/per-token scoping yet, so it can do whatever an `external:*`
  principal can do (send messages, spawn agents, ...); that gap is real, just not solved
  here. One-time setup: `bridle --url <daemon url> token create statusline > ~/.bridle/statusline.token` (`--url` so the token is printed, not saved in `credentials.toml`) to mint
  an `external:statusline` token and store it where `statusline` reads it (a fixed path under
  `$BRIDLE_HOME`/`~/.bridle`, not the workspace's own `.bridle/`, since this needs to work
  regardless of which project workspace Claude Code happens to be in).
- **`arch-guard`** is Claude Code's `PreToolUse` hook (`workflow/base/hooks/PreToolUse.json`,
  matcher `Edit|Write|MultiEdit`, rendered by `bridle sync`). It reads the hook JSON on stdin
  and, for an edit whose path (resolved lexically against `cwd`) is under `design/architecture/`,
  prints a `hookSpecificOutput` `permissionDecision: "deny"` unless the caller is not a worker
  agent or has claimed an `arch-revision` task; the reason tells it to run `bridle arch propose`
  ([[docs/design/architecture-tier|architecture tier]]). Any error of bridle's own allows.
- **`stop-check`** is Claude Code's `Stop` hook, registered only for the worker role
  ([[docs/design/coordination#How agents actually hear things (Claude Code integration)|coordination.md]],
  [[docs/spikes/05-stop-hook-findings|spike 05]]). It reads the hook's JSON on stdin; if
  `stop_hook_active` is set it allows immediately (Claude Code silently overrides a hook
  after 9 consecutive blocks in one turn, so a well-behaved hook blocks at most once per
  turn). Otherwise it lists the calling principal's own claimed tasks
  (`?claimed_by=me`) and blocks — printing the flat `{"decision":"block","reason":"..."}`
  spike 05 confirmed, not the `hookSpecificOutput` wrapper — on the first one with no
  thread entry (note, question or answer) from itself at or after `claimed_at`; naming
  the task and telling the agent to `bridle release` it or leave a `bridle task note`
  first. It also blocks when the tree looks finished (clean, commits ahead of local `main`/`master`)
  and a claimed task has no summary or no thread entry from itself starting `done:`, telling
  the agent to *run* `bridle task summary` and `bridle send ... done:` (workers were printing
  them). Before that report check, a finished-looking tree also has to have passed the
  project's `commands.check_worker` (falling back to `check`) at the current HEAD: the hook
  runs it itself when `<git-dir>/bridle-check-passed` doesn't name HEAD, records HEAD there on
  success (never committed; a later stop at the same HEAD skips the rerun), and otherwise blocks
  with the tail of the output. The hook is registered with a 30-minute timeout for this. Any error of bridle's own (unparseable stdin, no daemon reachable, an API
  error) allows rather than blocks: a bug in bridle's own tooling must never trap an
  agent from stopping.
- **`prime orchestrator`** prints a fresh orchestrator session's opening context in one
  go (docs/questions/open/one-command-orchestrator-handover-d4mz.md, step 2): the role
  prompt (`workflow/base/roles/orchestrator.md`, generic: `{project}` becomes the current
  project's name, the `--project` value or the directory name, e.g. in the credentials-entry
  snippet), followed by the project's own `.bridle/roles/orchestrator.md` when present, the
  newest handover note with its age (or, with none, `docs/context/orchestrator-state.md` when
  the project has one), then the startup steps (check in with
  `status`/`agents`/messages, start the watcher, keep the workforce's
  work moving, verify merges, watch context). The advisor role
  (`workflow/base/roles/advisor.md`) is generic the same way and points at an optional
  `.bridle/roles/advisor.md`. The note is fetched from the daemon
  best-effort (no daemon or no note: the state file); the files are read from the current directory, so run it from the repo root, as
  `scripts/claude-orchestrator` does when it uses this as `claude`'s opening prompt.
  Any other role is a clap `InvalidValue` error, not a silent no-op.
- **`prime worker|planner`** (planner = the `product-manager` rule tag) opens prime to
  those two roles, each for its own role only, to deliver component scope
  ([[docs/design/components|components]]). It prints the rules tagged for the role (or
  untagged), the facts and guide paths from L1–L3; then, for each component named by
  `--component` (repeatable) or else `$BRIDLE_COMPONENTS`, a section of its own: the
  chain's rules (only those a component layer defines, overrides or disables; the L1–L3
  ones are above), facts and guide paths, resolved separately per named component so two
  chains never merge, and docs pointers (the component's `docs` folder, which of
  README.md and roadmap.md exist, README.md inline when ≤40 lines; ancestors get none).
  A last line lists the components not named with their docs folders, which is what
  makes the scoping soft. Local, reads the current directory, renders nothing to files;
  an unknown component id is an error. `--task ID` (worker) fetches the task, and an `explore` one
  gets the exploring agent's paragraph first ([[docs/design/explorations|explorations]]). The role scope and the rest of the "commands still to build" surface (`init`) stays
  in `Planned` below; `sync` is built (see above).

## Planned

Commands for the phases after v1 ([[docs/proposal/build-order|build order]]),
as a first cut:

```
bridle task <cmd> at in_review|accepted          states not built yet
bridle handoff                                   bridle accept <id> (human only)
bridle inbox --inject
bridle spawn <role> <task>   bridle review
bridle take|give <agent>                         human takeover of a headless agent
bridle rules show|propose                        `explain`/`diff --project-layer` are built (see Built)
bridle trace coverage                            the other `trace` commands are built
bridle explore adopt                             `new`/`conclude`/`abandon` are built
bridle usage --by project|kind|task|trend|compare   `role|model|agent` are built
```

The command name and a short alias are open:
[[command-name-and-short-alias-sqt6|command name]].
