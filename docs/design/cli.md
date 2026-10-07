# The CLI

> **Status (checked 2026-10-03):** Built and in use: the commands under "Built", except the next ones · Built, not wired in: `workflow sync` and the `arch-guard` hook it installs (run by hand only; bridle's own repo never runs it, so its `.claude/settings.json` has no hooks), `orchestrator prime worker|planner` (no role, hook or skill runs it; the daemon puts the same resolved rules, without facts, guides or components, into every spawned agent's system prompt), `task ready --role` (accepted, ignored) · Planned: the "Planned" block at the end

Most commands are thin clients of the daemon's API
([[docs/design/agent-host/api|API]]); the local ones (`ticket`, `workflow rules|sync|spec|goals|arch|explore|trace`,
`orchestrator prime`, `hook`, `daemon init|doctor|launchd|systemd`, `completions`) say so below.
Every command takes `--json`, which agents always use; humans get compact tables.

## Grouping

Commands are grouped (ticket a67t): `daemon` (serve, stop, restart, doctor, init, launchd,
systemd, rebuild, list), `agent` (spawn, list, show, interrupt, stop, resume, renew, rm, logs),
`task` (the task records plus claim, release, ready, queue, dep, land, conflict, impact, ask,
answer), `usage` (the summary, plus `cost` and `budget`), `orchestrator` (note-session, handover,
prime, wait-for-wake), `workflow` (update, rules, sync, spec, goals, arch, explore, trace) and
the hidden `hook` (statusline, stop-check, arch-guard). `status`, `send`, `inbox`, `agents`,
`queue`, `events`, `wait`, `tui`, `token`, `port` and `probe` stay at the top level. Every old
top-level name (`bridle claim`, `bridle serve`, `bridle statusline`, `bridle stop-daemon`,
`bridle daemons`, ...) is a hidden alias that dispatches to the same code (`normalize` in
`commands/mod.rs`); hook settings, launchd/systemd units and scripts still spell the old names.
The aliases are dropped in a later release. The sections below use the grouped names.

## Project resolution

Every command that acts on a project picks it the same way (br-3397): `--project`, then
`$BRIDLE_PROJECT`, then the project of the workspace containing the cwd (`.bridle/daemon.json`
walking up, else a registered daemon whose workspace holds the cwd). There is no default project:
with none of the three, a command refuses and names `--project`. Commands that reach a daemon get
this from `discovery::resolve_endpoint` (after `--url` and `$BRIDLE_URL`); the rest (`session`,
`advisor`, `prime`, `ticket`, `launchd`, `systemd`) call `project::resolve` (`crates/bridle/src/project.rs`).
A command that acts on no project (or only on the cwd's repo) says so.

The rule is bridle's first spec, `design/specs/project-resolution.md`. Its executable scenarios
(`crates/bridle/tests/project_resolution_test.rs`) read every command and subcommand from
`--help` and run each from a temp workspace, with the flag, with the environment variable and
outside any workspace, checking which project it acted on. A new command must be classified in
that test's `CLASSES` table. `just check` runs `spec check` and `spec coverage` on bridle's own specs.

Also enforced by tests in `project.rs`: every top-level command in the clap tree must be listed in
its `SCOPES` table as daemon-backed, resolver-backed or no-project with a reason (a new command
fails until its author chooses), and no source file may hard-code a project name as a fallback.
`tests/session_test.rs` covers the behaviour: `bridle session` run from another project's
folder targets that project, and outside any workspace it refuses.

## Built

```
bridle [--url URL] [--project NAME] [--token T] [--json] <command>

bridle docs [TOPIC]                                             local, no daemon: with no topic, list the topics; with one, print its overview (embedded from `docs/cli/<topic>.md`, written for agents in any project)

bridle gateway [--detach]                                       run the human web UI's gateway, in the foreground or (`--detach`) in a new process group logging to `~/.bridle/gateway.log`, waiting for health and refusing if one already answers at `bind` (needs a fixed port); it re-executes itself when its binary is replaced; `[gateway] enabled = false` makes it exit 0 without starting (docs/design/human-web-ui.md, "Running it detached"); `[gateway]` in
                                             `~/.bridle/config.toml`: `bind` (default `127.0.0.1:7878`; `0.0.0.0` refused unless `allow_any_interface = true`).
                                             Only this command reads that section, so a bad one fails here and nowhere else. `GET /api/v1/health`
                                             is open; `username` + `password_hash` (argon2) enable `POST /api/v1/login` (JSON `username`, `password`; sets an `HttpOnly`,
                                             `SameSite=Strict` session cookie) and `/logout`; every other `/api/v1` route needs that session, and with no login configured only health answers.
                                             Every other path serves the UI folder (`ui_dir`, default `~/.bridle/ui/`; open, so the login page can load; index fallback; no `..`); its `api-version` file is checked against the gateway's, `ui_version_mismatch` = `warn` (default; logged, shown in health's `ui`) or `refuse` (503 at `/`)
bridle gateway hash-password                                    read a password, print its argon2 hash for `[gateway] password_hash`: when stdin is a terminal, prompt "Password:" on stderr, read with echo off, ask to confirm, refuse on mismatch; when stdin is not a terminal (piped), read the line without prompt for script compatibility
bridle gateway install [--force]                              write a launchd plist (macOS, `~/Library/LaunchAgents/dev.bridle.gateway.plist`) or systemd user unit (Linux, `bridle-gateway.service`) that runs `bridle gateway` at login/boot, restarts it on a crash only, and logs to `~/.bridle/gateway.log`; prints the load/unload commands, never runs them; needs a `[gateway]` login first; refuses to overwrite without `--force`
bridle sign setup | binary [path]                             macOS: `setup` creates the self-signed "bridle local signing" identity in the login keychain (asks the keychain password once) and sets the key's partition list so codesign needs no prompt; uses the system /usr/bin/openssl (LibreSSL) because Homebrew's OpenSSL 3+ writes a p12 that macOS `security` cannot import; `binary` re-signs a binary (default: this one) with it, or keeps the ad-hoc signature when the identity is absent; `just install` and the self-upgrade call it (p88z)
bridle daemon serve   [--repo PATH] [--workspace DIR] [--listen ADDR] [--detach] [--take-over]   --take-over: claim a project another host owns; refuses (both SHAs named) unless
                                              origin was reached and bridle/state + the integration branch fast-forward cleanly
                                              listens on: `--listen`, else `[daemon] listen`, else the `[projects]` port for this project when it is on this
                                              machine (127.0.0.1 + the Tailscale IPv4 address, never 0.0.0.0), else 127.0.0.1:0
bridle daemon stop                            prints "requested shutdown", "acknowledged; the daemon is stopping N agents,
                                              up to Ns" (the daemon's stop_grace + 5 s), "N agents still running" as the count drops,
                                              then "shutdown complete (Ns)"; after 60 s it errors, pointing at `bridle daemon list`
                                              and <workspace>/.bridle/daemon.log
bridle daemon restart [--wait SECS] [--upgrade]                restart the daemon in place once every agent is idle (orchestrator or human); prints the commit and the agents to
                                              resume, then "the daemon is back". A busy daemon (nothing idle within --wait, default 600) errors and stays up. --upgrade first builds the newest green-CI commit on main (background; prints "building <sha>" or "nothing to upgrade" and returns; the daemon restarts itself after the build)
bridle daemon init [--repo PATH] [--name N] [--integration BRANCH] [--stack S]  scaffold .bridle/config.toml + .gitignore; never overwrites. A project with no `workflow` (and no `workflow/base/` in the repo) also gets the base workflow vendored into `.bridle/workflow/` (uncommitted; you commit it): copied from the clone this binary was built from if it's still there, else `git clone --depth 1 --branch v<version>` of `workflow_url` (machine `~/.bridle/config.toml`; default the bridle GitHub repo). A fetch failure is an error.
bridle workflow update [--repo PATH] [--to TAG]   re-fetch the vendored `.bridle/workflow/` (from the local clone, or the tag: `--to`, else this binary's) and print added/changed/removed files; the only thing that ever changes it. Errors if the project isn't vendored.
bridle daemon doctor  [--repo PATH]                 check the project's setup, say what to fix; exit 1 on a failure
bridle daemon launchd install [--repo PATH] [--workspace DIR] [--force]   macOS: write the LaunchAgent plist, print launchctl commands
bridle daemon launchd uninstall                    remove the plist, print the bootout command
bridle daemon systemd install [--project P] [--projects-dir DIR] [--force]   Linux: write a systemd user unit per project `[projects]` puts on this machine, print the systemctl and linger commands
bridle daemon rebuild [--from-origin]              first fetches origin/bridle/state (fast-forward only); reconstructs tasks/edges/open_questions/claims
                                              (claims.toml) from the state branch alone; the migration path for a fresh
                                              clone with no bridle.db yet; also restores the handover notes (handovers/<id>.md)
bridle daemon list                              # every running project daemon on this machine, with agent counts
bridle status                               # daemon, agents, active incidents, Claude Code version, the last wake delivered and whether a waiter is open, last CI result (sha, conclusion, age, url) when [ci] github is on; state branch push (age, or the failure) when [state] push is on
bridle agent spawn   <role> [--name N] [--prompt TEXT | --prompt-file FILE]
               [--worktree [--base REF] | --in-repo | --cwd PATH] [--model M]
               [--allow-tool TOOL ...] [--env KEY=VALUE ...] [--ignore-budget]
               [--component ID ...]
bridle agents  [--all]
bridle agent show    <agent>
bridle send    [--project <other>] <agent|human|role:NAME|external:NAME> [TEXT | --text-file FILE] [--question] [--when now|idle] [--reply-to ID] [--task ID]
bridle inbox   [--all] [--mark-read]        # messages to me, plus every task's open question (list); an agent's or external principal's listed messages are marked read
bridle inbox show <id> [--mark-read]        # show one message in full; for the human it leaves it unread unless --mark-read, for an agent or external principal it marks it read
bridle inbox read <id>...                   # mark one or more messages read
bridle inbox unread <id>...                 # mark one or more messages unread again (human only; others get 403)
bridle task ask     <task-id> TEXT [--to WHO]         question against a task; blocks it until answered, and sends a pointer message (kind question) to WHO (agent, role:NAME, external:NAME, human); default: the caller's spawner, or human
bridle task answer  <task-id> TEXT                    answers a task's open question; frees it to be ready again; sends the asker a pointer (kind answer)
bridle task claim   <task-id>                         claims a ready task for the caller: planned -> claimed
bridle task release <task-id>                         releases the caller's own claim: claimed -> planned
bridle task ready   [--all] [--role R]                the highest queue tier with a startable task (planned, deps met, no open question, unclaimed)
bridle task ready   <id>                                 pending -> open: approve the task for the PM (orchestrator or an advisor with the human's approval; a manager for its own small fix inside approved work; not workers, the PM or visitors). The daemon messages the PM (else the orchestrator) that it is open, and returns it to `pending` with a note if nobody plans it within `[tasks] open_stale` (default 4h). `bridle status` lists pending tasks
bridle queue                                     read-only: claimed tasks with their worker, then the tiers in rank order
bridle queue set --tier T,T... [--tier T,T...]   replace the whole queue, one --tier per tier (PM, orchestrator or human only)
bridle queue add-tier <task>...                  append one tier at the back (PM, orchestrator or human only)
bridle task dep add|rm <task> (--to OTHER [--kind K] | --blocked-by OTHER)   K: blocks (default)|parent|discovered-from|related|supersedes|duplicates
bridle wait    <task> [--until STATE] [--or-message] [--timeout SECS]   block until the task changes state; exit 4 on timeout
bridle agent wake <identifier> [--timeout SECS] | --stop [<identifier>]   (cap and default 6900 s = 1 h 55 min) blocks until the daemon decides that principal should wake (reason: it has an unread message, which includes the `task_update` lines about tasks it watches); prints each message (id, sender, text) (`--json`: `{reasons:[{reason,message_ids,messages,task?,event?}]}`); a non-human caller's messages are marked read by the call, the human's are not; exit 0 woken, 4 timed out, 5 superseded (a newer wait from the same session, `BRIDLE_SESSION_PID`, replaced this one, or `--stop` ended it; reason `superseded`; nothing marked read; a wait with no session replaces nothing), 6 the daemon is restarting or shutting down (reason `daemon_stopping`, `text` says why; stderr); it prints `waiting as <identity> (pid N, timeout S s)` on stderr first; `--stop` ends this session's open wait through the daemon, or with an identifier that identity's waits (own only; the human any), printing `stopped N wait(s)` or `no wait open`; caller must be that principal (or the human), else 403
bridle agent interrupt <agent> [--drop-held]
bridle agent stop    <agent> [--now]      bridle agent resume <agent> [--ignore-budget]
bridle agent renew   <agent> [--ignore-budget]    stop + fresh process/session, same worktree/branch/role/model
bridle agent rm      <agent> [--force] [--delete-branch]
bridle agent logs    <agent> [--follow] [--raw] [--since LINE]
bridle events  [--follow] [--since SEQ] [--agent A] [--kind PREFIX]
bridle usage   [--by role|model|agent] [--since DURATION]   # DURATION: <n>s|m|h|d, e.g. 30d
bridle usage --history WINDOW [--since DURATION]            # the window's rate-limit readings over time (five_hour, seven_day)
bridle usage cost audit [--check]                 static: size of what bridle injects into agent context (usage-and-budget.md)
bridle tui                                  interactive terminal UI: agents list, live event tail
bridle usage budget [--schedule]                  usage governor status, or the whole resolved schedule (usage-and-budget.md)
bridle usage budget hold [--for D | --until HH:MM] | release          idle the account for the human / end the hold early
bridle usage budget override <period|default> [--until HH:MM] | override-clear   force a schedule period's thresholds / revert
bridle usage budget max-workers <N> | --clear     live worker cap, lost on daemon restart
bridle token create <name> [--print]        with a known project (--project, or the cwd's daemon) the token is saved in
                                             ~/.bridle/credentials.toml and not printed unless --print; with --url it's
                                             printed once. A name with `@` is refused
bridle token create <name> --machine <m>    a visitor, `external:<name>@<m>`, for a principal on another machine: always
                                             printed once, never saved here; paste it into that machine's credentials.toml
bridle token create --peer <m>              a peer token, `peer:<m>`, for the daemons of machine <m> to forward mail here: always
                                             printed once; paste it under `[peer]` in the sender's credentials.toml, keyed by this project
bridle token list                           name, created-at, revoked-or-not; never the token itself
bridle token revoke <name>                  human only, external tokens only (an agent's own token is
                                             revoked through `bridle agent rm`, not this); also removes its
                                             credentials.toml entry for the project
bridle hook statusline                      Claude Code statusLine command; local only, no daemon call
bridle completions <zsh|bash|fish|elvish|powershell>   print the shell completion script generated from the clap definition, never drifting from the CLI. Static completions only (subcommands, flags, enum values like `--kind`). Install once with `bridle completions zsh > ~/.zfunc/_bridle` (and add `fpath=(~/.zfunc $fpath)` to ~/.zshrc); bash via `bridle completions bash > ~/.local/share/bash-completion/completions/bridle`; local only, no daemon call
bridle migrate [--dry-run] [--project NAME | --all]   apply the project migrations this bridle ships that the project hasn't had yet, in order, once each ([[docs/design/migrations|migrations]]). Default: the current repository; `--all`: every registry project, one at a time, stopping at the first failure and naming the ones done. `--dry-run` prints what would change and writes nothing. Never runs by itself. Records in the project (`.bridle/migrations.toml`, `.bridle/migrations.log`) and, if its daemon answers, as a `project.migrated` event
bridle orchestrator note-session            the orchestrator launcher's SessionStart hook: writes $BRIDLE_HOME/orchestrator.session
                                             from the hook JSON on stdin; local only; never fails
bridle review add|remove|list <path>          documents under review (x8jt): edits `.bridle/review-documents.txt`; the daemon wakes a document's
                                             agent on new comments (docs/design/agent-host/daemon.md, Document review)
bridle review resolve <path> c3               closes a comment thread: appends `resolved by human, <time>` to it in the file (ticket ehv6)
bridle review now <path> [--resend]           sends the document's pending comment threads to its agent at once, skipping the quiet period;
                                             marks them `[sent YYYY-MM-DD HH:MM EDT]` in the file; marked threads stay out unless --resend
bridle link <ID>                                                local, no daemon: the bridle UI URL for a ticket ID (`<base>/ticket?project=<p>&id=<id>`) or a task ID with a `-` (`<base>/task?id=<id>`), from `[gateway] public_url` (project config over machine config); prints nothing and exits 0 when unset (yfjc)
bridle focus gate                           the UserPromptSubmit hook of focus hours (cvaq): in a `quiet` `[[focus]]` period prints
                                             nudge context on the first prompt and every 5 min after; silent otherwise; never fails
bridle orchestrator handover done                       the orchestrator's state is written: the daemon stops and relaunches its session (marker only); human and external:orchestrator only
bridle orchestrator wait-for-wake --mail [--timeout SECS]                  the advisor's mail-only waiter: returns when unread mail from external:mail arrives (`nothing` at the timeout, default 25 min, cap 6900 s); polls the inbox every 10 s
bridle handover write --file <path>|-                  record your handover note (any principal; keyed by your identity and the project); prints its id
bridle handover list [--role R] | show <id> | latest [--role R]     the notes, newest first · one note · the newest
bridle mail run                              the email bridge for this project: inbound mail, question mails, daily digest (docs/design/mail.md); runs as external:mail
bridle orchestrator wait-for-wake [--timeout SECS]                  the orchestrator's background watcher: waits for a wake condition, prints it and exits 0 (6 with the reason on stderr when the daemon is restarting or shutting down; `nothing` at the timeout, default 25 min, cap 6900 s); external:orchestrator only
bridle hook arch-guard                      Claude Code PreToolUse hook: blocks design/architecture/ edits outside an arch-revision task
bridle hook kill-guard                      Claude Code PreToolUse hook (Bash): refuses kill-by-name (pgrep | xargs kill, kill $(pgrep), pkill in a compound command)
bridle hook stop-check                      Claude Code Stop hook for the worker role; refuses to stop
                                             with an unreleased claim and no thread entry since claiming
                                             it (docs/design/coordination.md); never fails
bridle workflow rules explain <id>                   which layer wins a rule id, and what it shadowed
bridle workflow rules diff --project-layer           everything the project layer does differently from
                                             the base/pack layers below it
bridle workflow rules explain|diff ... --component <id>  the same on top of that component's chain (L4,
                                             root-most ancestor first); diff shows what each
                                             component layer changes instead of the project layer
bridle workflow sync                                 renders resolved workflow layers into CLAUDE.md's
                                             managed block, .claude/skills, .claude/agents and
                                             .claude/settings.json's hooks; local only, no daemon call
bridle workflow arch list [--invariants] [--root DIR]   lists architecture elements (id, `invariant` flag, title,
                                             file:line) from `*.md` under DIR (default
                                             `design/architecture`); a missing or duplicate `a-` id
                                             is an error (diagnostics on stderr, exit 1); --json
                                             prints the elements with their text; local only
bridle workflow arch propose --title T (--argument TEXT | --argument-file FILE|-) [--arch-root DIR]   creates an `arch-revision` task with the proposal
                                             (daemon call); validates that the architecture
                                             directory exists locally
bridle workflow trace down|up <id>  [--goals DIR] [--arch DIR] [--specs DIR]   walks the trace links across
                                             `design/goals`, `design/architecture` and `design/specs`:
                                             `down` lists everything depending on the element, `up` what it
                                             rests on, up to the goals (indented by distance; --json gives
                                             rows with `depth`); an unknown id, an unknown link target or
                                             any parse error is an error (exit 1); local only
bridle workflow trace orphans                         requirements with no `traces=` (a warning on stderr, exit 0)
bridle workflow trace suspect                         links whose recorded `@hash` differs from the upstream element's
                                             current hash (id, file:line, upstream, recorded, current; --json
                                             prints them); exit 1 if any
bridle workflow trace confirm <id>                    rewrites that requirement's link hashes to the current ones; edits
                                             only the heading line, the rest of the file byte-for-byte
bridle workflow explore check [paths...]              checks exploration findings frontmatter (default `design/explore`);
                                             diagnostics on stdout, exit 1 on any error; local only
bridle workflow explore new|conclude|abandon <id>     scaffolds `design/explore/<id>/findings.md` (status open;
                                             refuses to overwrite) or rewrites just its `status:` line
bridle pane tag <name>                       set the @bridle tmux pane option to the given name (errors
                                             if not in a tmux pane); used by launcher scripts
bridle pane untag                            clear the @bridle tmux pane option (errors if not in a
                                             tmux pane)
bridle machine tools-only-check [--repo P]   exit 1, saying why, if the clone is listed in ~/.bridle/config.toml
                                             `[machine] tools_only = [paths]` (~ and $VAR expand); the
                                             launch scripts call it. `bridle daemon serve` refuses there too
bridle machine tools-only-install [--repo P] install pre-commit and pre-push hooks that refuse in a
                                             tools-only clone (`--no-verify` overrides); re-runnable;
                                             moves a hook that isn't bridle's to `<hook>.pre-bridle` (refuses if that
                                             exists); once the clone is no longer listed, re-running removes bridle's
                                             hooks and restores those, keeping both if a different hook appeared (hw6c, ged2)
bridle workflow spec check [paths...] [--root DIR] [--require-ids]   validates spec files (dirs are searched for
                                             *.md; default `design/specs`, or --root) with the
                                             bridle-spec parser: prints file:line:col: message per
                                             diagnostic and a summary, exit 1 on any error; a
                                             requirement without an id is a warning (an error with
                                             --require-ids); --json prints them as structured output;
                                             local only, no daemon call
bridle workflow spec id [paths...] [--root DIR] [--ledger FILE] [--dry-run]   writes a stable id (`{#r-xxxx}` /
                                             `{#s-xxxx}`) into every requirement and scenario heading
                                             lacking one, in place, touching only those heading lines;
                                             ids are unique across the files processed and never reused
                                             (ledger `<root>/.ids`); idempotent; `--dry-run` prints
                                             the plan and writes nothing; local only, no daemon call
bridle workflow spec export --format gherkin|json [--out DIR] [paths...] [--root DIR] [--scenario ID]... [--task ID]
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
bridle workflow spec coverage [--root DIR] [--tests DIR ...] [--require-all] [--json]
                                             lists executable scenarios whose id does not appear in
                                             test sources; scans text files (skip binary, node_modules,
                                             target, .git) under --tests directories (default `tests`
                                             and `test` if present) for the token 's-' plus hex ids
                                             of scenarios; outputs counts (executable, bound, unbound)
                                             and the unbound list as 'file:line: s-id title'; exits 1
                                             with --require-all if any unbound (default exit 0); local
                                             only, no daemon call
bridle ticket new "<title>" --kind <kind> [--repos a,b] [--needs ids] [--see ids] [--body t | --body-file f]   mints
                                             `docs/tickets/open/<slug>-<id>.md` (repo root found via git): a
                                             fresh 4-character ID unique across `open/` and `resolved/`,
                                             frontmatter id, title, kind (required, no default: the task kinds), opened (UTC date), repos (default: the
                                             project name), changes, specs, needs, see, tasks, and an empty
                                             `## The ask`; creates the folders if missing; prints the path.
                                             `--body`/`--body-file` (`-` = stdin) write the ask into the stub. Files no
                                             task (the task would race the ticket's commit; k7tm); `--no-task`
                                             is a hidden no-op. The new ticket's id is never a task's id either
                                             (one id space)
bridle ticket new --from-task <task-id> [--kind k] [title]   as `ticket new`, from an existing task (needs a daemon):
                                             title and kind default to the task's; the ticket takes the
                                             task's id, or a fresh one when the task's tail has a character
                                             the ticket alphabet lacks (old hex ids with 0/1) or a ticket has
                                             it; either way the ticket's `tasks:` lists the task and the
                                             task body gets the `original id: <ticket>` first line
bridle ticket task <id>                      files the task for an open ticket (its title and kind, body
                                             `original id: <id>` and the path) and records its id in the
                                             ticket's `tasks:` (the first task from a ticket takes the ticket's id,
                                             `br-k7tm` for `k7tm`; later ones get fresh ids). Refuses an empty
                                             `## The ask` and a ticket file not in the tip of local `main`
                                             (commit it first); needs a daemon; prints the task id
bridle ticket resolve <id>                   stamps `closed: <UTC date-time>` into the frontmatter and moves
                                             the ticket from `open/` to `resolved/` (a plain move: committing
                                             is the caller's); errors on an unknown or ambiguous id; doesn't
                                             touch the task. Both are local file work, no daemon start-up
bridle ticket set <id> <field> <value>       edits one frontmatter field of an open or resolved ticket: `title`,
                                             `kind` (any time; independent of its tasks' kinds), or a list field (`repos`, `changes`, `specs`, `needs`, `see`, `tasks`) given
                                             comma-separated (empty clears it); refuses `id`, `opened`, `closed`
                                             and unknown fields. Local file work
bridle ticket check [--quiet]                checks every ticket in `docs/tickets/{open,resolved}`: all of id,
                                             title, opened, repos, changes, specs, needs, see present; `closed`
                                             present under `resolved/` and only there; the id matches the file
                                             name's tail and is unique; `needs`/`see` name existing tickets (id or
                                             full stem); `[[stem|text]]` links outside code fences point at a file
                                             (stem anywhere under `docs/`, or a path from the repo root or
                                             `docs/`); `kind` is a task kind; `tasks` and the tasks' `original id:` agree (task side checked only
                                             when the daemon is up; a missing `kind`/`tasks` is a warning for now, see
                                             `MISSING_KIND_OR_LINK_IS_ERROR` in `ticket.rs`). Problems go to stderr, one per line, exit 1; `--quiet`
                                             prints nothing when clean. Local, no daemon
bridle ticket submit -k <kind> <title> [--body S | --body-file F|-]
                                              files a `pending` task on the project's daemon (any principal with a token,
                                              visitors included; no ticket file) and prints its id. The body starts
                                              `submitted by <principal>`; the project manager (else the orchestrator)
                                              gets one inbox message. Dropping it with a reason tells the submitter.
bridle workflow goals list [--root DIR] [--priority P] [--stance S]   lists goals (docs/design/goals-tier.md) from
                                             `*.md` under --root (default `design/goals`): id, firmness,
                                             priority, stance, title per line; the stance is defaulted from
                                             the priority when not written; --json prints the goals and
                                             diagnostics; diagnostics go to stderr as file:line:col: message
                                             (an unaddressed goal without a why is a warning); exit 1 on
                                             any error; local only, no daemon call
bridle workflow goals propose <goal-id> --change KEY=VALUE --why TEXT [--goals-root DIR]  creates a task proposing a change to the goal's
                                             firmness, priority, or stance (repeatable --change;
                                             daemon call); validates that the goal exists
bridle workflow spec import openspec [--from DIR] [--to DIR] [--dry-run]   moves each `<from>/<cap>/spec.md`
                                             (default `openspec/specs`) to `<to>/<cap>.md` (default
                                             `design/specs`) with `git mv` (plain rename if untracked),
                                             then assigns ids as `spec id` does; parses first and
                                             changes nothing on any error or existing target;
                                             `.feature` files, changes, config left in place;
                                             idempotent; local only, no daemon call
bridle orchestrator prime orchestrator                   fresh session's opening context: role prompt, current
                                             state, startup steps; local only, no daemon call
bridle orchestrator prime advisor                        the advisor role file (workflow, then .bridle/roles/advisor.md); local
bridle orchestrator prime aide                        the aide role file (workflow, then .bridle/roles/aide.md); local
bridle session orchestrator [claude args]   start the orchestrator's claude session from any directory: lean
                                             settings, names orch-<project>[-<BRIDLE_SESSION_SUFFIX>], pane tag,
                                             pid/exit files; project per Project resolution (no default); refuses under a bridle
                                             agent (unless BRIDLE_LAUNCHER_TEST=1) and in a tools-only clone
                                             (all `bridle session` roles also merge the workflow layers' hooks into
                                             --settings, e.g. base's prompt time stamp; workflow-layers.md; they also set
                                             autoMode.environment: `$defaults`, a derived workspace line, then `[auto_mode]
                                             environment` lines: trust lines from ~/.bridle/config.toml only, and from
                                             the project's .bridle/config.toml only lines starting `Sensitive:` or
                                             `Prod host:`, others are dropped; never writes ~/.claude/settings.json)
bridle session advisor [name] [claude args] same for the advisor (advisor[-<name>]-<project>): lean
                                             settings, pane tag (advisor or advisor-<name>), sets BRIDLE_ADVISOR_NAME;
                                             the unnamed one keeps advisor-<project>.pid while it runs; refuses
                                             (and names pid, pane, machine and `bridle session restart <identity>`)
                                             when a session of that identity is registered with a live process in
                                             the project; a registered one whose process is gone does not block
bridle session aide [claude args]         start the aide session (aide-<project>): lean settings, pane tag
                                             `aide`, BRIDLE_AS=aide so it signs `external:aide` (token under
                                             [aide]). It talks with the human about the running system; the
                                             orchestrator reaches the human only by messaging it. Not registered
                                             with the daemon; refuses while focus hours are locked, and while a live
                                             `aide` session is registered (as for the advisor)
bridle advisor start <name> [--brief TEXT|@FILE]   send the brief to external:advisor as "For advisor <name>: ...", then run
                                             `bridle session advisor <name>` in a tmux pane: a split of the pane tagged
                                             @bridle=orchestrator, else a new window (`[tmux] advisor_pane = "split"|"window"`
                                             in ~/.bridle/config.toml). Outside tmux prints the command, exit 0. Orchestrator
                                             and human only (refuses under BRIDLE_AGENT_ID)
bridle orchestrator prime prototyper         the prototyper role file, then the project's .bridle/roles/prototyper.md
bridle orchestrator prime document-reviewer  the document-reviewer role file, then the project's .bridle/roles/document-reviewer.md
bridle orchestrator prime worker|planner [--component ID ...] [--task ID]   the role's rules, facts, guides, plus named components' scope; --task is worker only
bridle task new    <title> -k/--kind KIND [--body TEXT | --body-file FILE] [--component ID ...] [--size S|M|L] [--for-human] [--priority critical|urgent|high|normal|low]
bridle task show   <id>
bridle task plan   <id>                                                 open -> planned: ready to build, claimable once unblocked
bridle task priority <id> critical|urgent|high|normal|low  change the priority; who and when go in the thread and a `task.priority` event
bridle task kind <id> <kind>             change the kind, only while `pending` or `open` (refused planned, claimed, dropped, integrated, reopened); who and when go in the thread and a `task.kind` event
bridle task edit   <id> [--title TEXT] [--body TEXT | --body-file FILE] [--component ID ... | --no-component] [--size S|M|L|none]
bridle task list   [--claimed-by WHO] [--component ID] [-k KIND]             WHO: me|human|<agent name>|<principal id>; unclaimed tasks have no claimant to match
bridle task search <words...>                                      search for tasks by words in title/body/summary (case-insensitive substring match, all words must match); includes done and dropped tasks
bridle task drop   <id> --reason TEXT
                                                                        incidents are `-k incident` tasks (`task list -k incident` lists them); plan/done/drop/reopen of one is orchestrator/human only, and `plan` sends the notice to every agent; see agent-host/incidents.md
bridle task done   <id> [--commit SHA] [--branch NAME] [--resolution TEXT]  -> integrated; `--commit` is required unless the human claimed the task (a to-do) or the task is an incident (`--resolution` goes in its thread and the "resolved" note); records the sha (and branch) on the task and in the thread; with --branch removes the branch's agents, worktree and branch; warns if no summary
bridle task impact set  <task> [--modify ID].. [--add-under ID].. [--remove ID].. [--files GLOB..]  declares the task's impact, replacing any earlier one; only a pending/open/planned/claimed task; ids checked by shape (r-/s-/g-/a- + hex) only
bridle task impact check [--specs DIR]                                overlaps between in-flight tasks' declared impact, plus merge probes of claimed tasks' branches (`--json`: `{overlaps:[{level,tasks,kind,key}]}`); exit 1 if any is a conflict; see impact-and-conflicts.md
bridle task land <task> [--branch B] [--check-cmd CMD] [--checked-commit SHA]              the integrator: squash the branch into one commit (`<task id>: <title>`, the summary as body, `Task:`/`Branch:` trailers) in the integration worktree, run `[integration] check` (skipped, with a note, only when the integration branch is an ancestor of the branch tip and the tip is `--checked-commit`, the commit the worker reported a green check on; otherwise it runs and the note says why), fail if the nextest test count is outside the sane band around the last full run (`<workspace>/last-full-test-count`), fast-forward the integration branch (guarded), mark the task done; any failure lands nothing (exit 1); never pushes
bridle probe <task-or-agent> | --branch B                       `git merge-tree` of the branch into the integration branch, no working-tree change (`--json`: `{branch,against,outcome,paths}`); exit 1 on a conflict; needs git 2.38
bridle port alloc [--pid N] [--label L]                         allocate a free port from `[ports] range` (not reserved, allocated or listening); prints it (`--json`: the allocation); recorded against you and your claimed task
bridle port release <port>                                      free a port
bridle port list                                                allocated ports (`--json`: array of `{port,agent,task,pid,label,allocated_at}`)
bridle task conflict list                                            conflicts opened by `impact check`, open first (`--json`: array of `{id,tasks,kind,key,state,resolution,opened_at,resolved_at}`)
bridle task conflict resolve <C12> --compatible <reason> | --order A,B | --merge-into <task>   record the outcome (exactly one flag); `--order` adds an `A blocks B` edge; A, B and the task must be the conflict's two tasks; a resolved conflict can't be resolved again
bridle task impact show <task>                                         prints the declared impact (`--json`: the impact object)
bridle task summary <id> --text TEXT | --file FILE                    records how it was implemented; `-` reads stdin; replaces an earlier summary
bridle task reopen <id>
bridle task watch <id> · unwatch <id>                               add/remove yourself as a watcher (`task show` lists them; the creator and claimer are added automatically)
bridle task skip-settle <id> --reason TEXT                            skip the settle period (recorded); human, or orchestrator/PM with a reason saying the human asked or an urgent downtime fix
bridle task comment <id> [TEXT | --text-file FILE] [--notify AGENT]  plain comment on the task's thread; tells the watchers and the claimant (not the author); no effect on readiness
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
  the slid `max_staleness` (up to 6x while usage is low; usage-and-budget.md). `--schedule` prints the whole resolved schedule instead.
  Text times are local machine time; `--json` keeps UTC/ISO 8601
  ([[docs/design/usage-and-budget#Seeing what applies|details]]).

- **`send`**: an empty or whitespace-only text is refused (by the CLI, and by the daemon
  with 400), here and in `task comment`. When given `--text-file FILE` or `--prompt-file FILE`, pass `-`
  as the filename to read from stdin instead. This avoids passing backticks and
  other shell metacharacters as command-line arguments, which can trigger
  permission denials in Claude Code. Example: `echo "message" | bridle send w1 --text-file -`.
  `role:NAME` fans the message out to every live agent currently holding that
  role — one delivered message per matching agent, same as sending to each
  individually; `external:advisor/<name>` (`@machine` allowed) addresses one advisor session: if
  it isn't running the message goes to `external:advisor`, marked "(originally for advisor/<name>)",
  and `send` adds "<name> isn't running; delivered to advisor"; `bridle send` prints one `sent <id> -> <to>` line per recipient.
  A role with no live agents is an error, same as an unknown agent name.
  `--project <other>` naming a different daemon than the sender's own (3haz): the CLI hands the message to
  the sender's own daemon (the cwd's, or `$BRIDLE_URL`'s), which queues it in its outbox and prints
  `queued o-0007 for <project> -> <to>` at once, then forwards it (principals.md, "Mail between
  daemons"). It never writes to the other daemon, and succeeds while that daemon is down. The
  recipient is named as on the other daemon; `--task` isn't supported that way yet. With no own
  daemon to hand it to, the old direct send to `--project` applies. The own daemon's project is the
  cwd workspace's; a daemon found by `$BRIDLE_URL` is asked (`status`), and one that can't say counts as
  the same project, so an agent's own `$BRIDLE_URL` + `$BRIDLE_PROJECT` sends locally, `--task` too (x56y).
  `--task <id>` (and `bridle task comment <id> --notify <agent>`, the same call) writes the text as a comment on the task's thread and sends the recipient `<id>: comment added` plus its first line; an unknown task is an error and nothing is sent.

- **`task new/edit/comment`**: when given `--body-file FILE` or `--text-file FILE`, pass `-`
  as the filename to read from stdin instead. This avoids passing backticks and
  other shell metacharacters as command-line arguments, which can trigger
  permission denials in Claude Code. Example: `cat long-body.txt | bridle task new "title" -k feature --body-file -`.
  `--body-file` is mutually exclusive with `--body`; `--text-file` is mutually exclusive with the positional `TEXT` argument.

- **`rebuild`** is `TaskManager::rebuild_from_state_branch` (docs/design/storage.md,
  "Rebuild"): the migration path for a fresh clone with no `bridle.db` — clone the repo,
  start the daemon, `bridle daemon rebuild`. `--from-origin` first fetches `origin/bridle/state`
  (`POST /v1/rebuild?from_origin=true`; fast-forward only, never overwrites a local branch with
  state of its own, and says what it did). Human-only; refuses (409) rather than overwrites if
  the database already has any tasks, edges or open questions. Claims are never
  reconstructed — they're SQLite-only, with no state-branch counterpart, so any in-flight
  claim is simply lost, which is correct here, not a gap.
- **Landing record.** `task done --commit SHA [--branch NAME]` stores `commit` and `branch` on
  the task; `task summary <id>` stores a short implementation summary (any state; a second call
  replaces it). `task show` prints the creator (`created by`; `unknown` for an old task whose creator was never recorded), and prints branch, commit and summary together, so a task id leads to
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
- **`--priority critical|urgent|high|normal|low`** on `task new` (default normal) and `task priority <id> <p>`
  set the priority. `task list` and `task show` display it, and `task list` sorts by it,
  highest first. At `high` and above the most recently set goes first (so a newer urgent thing
  outranks older ones at its level, and an agent putting things "at the top" is outranked by
  the next one to do so); at `normal` and `low` it is oldest first. The order is one function
  (`sort_by_priority` in bridle-api) shared by the CLI and the gateway's to-do list, so `task list --claimed-by human` is the human's ranked
  list. `task drop --reason` on a to-do the human holds also sends the human an inbox note
  with the reason.
- **`task edit --body`** keeps a ticket-born task's first line `original id: <ticket>` (the link
  `ticket check` reads) when the new body doesn't start with its own such line; a body that does
  wins.
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
- **Projects on other machines** (k7mw): `~/.bridle/config.toml`, the same on every box and
  trusted (no probing), names this machine (`[machine] name = "mbp"`, a key of `[machines]`),
  the host to reach each machine by (`[machines] nuc = "nuc"`) and where each project lives
  (`[projects] meta-notes = { machine = "nuc", port = 7402 }`). `--project`/`$BRIDLE_PROJECT`
  naming a project on another machine goes to `http://<host>:<port>` and takes its token from
  `[principal.<machine>]` in `credentials.toml`; a project on this machine, or not listed, uses
  the local registry as before. A listed project with no `[machine] name` is an error. Types:
  `bridle_api::machines`.
- **Exit codes**: 0 ok, 1 error, 2 usage error, 3 daemon unreachable (every
  discovery failure, including an unknown `--project`), 4 `wait` timed out, 5 an `agent wake` was superseded by a newer wait from the same session or stopped with `--stop`, 6 a waiter
  (`agent wake`, `orchestrator wait-for-wake`) was ended because the daemon is restarting or shutting
  down: the reason is on stderr; re-arm once it is back (`--json` still prints the wake on stdout).
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
  `bridle_daemon::rules`). The same resolution (L1-L3, no components) goes into every
  daemon-spawned agent's system prompt as `## Workflow rules`
  ([[docs/design/agent-host/roles-and-config|roles and config]]), so what `explain` reports is
  what a spawned agent is told. They resolve L1 base, L2 packs and L3 project rule layers
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
  with different value types. The L1 base and L2 pack layers come from the top-level
  `workflow = "path"` (relative paths resolve against the repo root; the machine config's
  `workflow` wins, roles-and-config.md) and `packs = ["name", ...]` in `.bridle/config.toml` (or, with no `workflow`, the copy `bridle daemon init`
  vendored into `.bridle/workflow/`). With none of those, resolution sees only the project layer.
  A `workflow` directory that doesn't exist, or a git url, is an error (`Config::workflow_root`);
  a listed pack whose directory is missing loads as empty (`bridle daemon doctor` flags it).
  Packs read `<workflow>/packs/<name>/rules` in listed order; `workflow/packs/` has `python`,
  `typescript` and `vim`. Bridle's own workflow lives in-repo (`workflow = "workflow"`,
  [[where-bridle-workflow-lives-r2uq|r2uq]]).
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
  component rules (`bridle orchestrator prime worker --component` prints them; nothing delivers
  them to agents yet, docs/design/components.md), same as `rules explain`/`diff` above. Nothing
  runs `sync` automatically (a `SessionStart` hook is not built): it's a command you run
  yourself, and bridle's own repo doesn't. Its output reaches a spawned agent only through what
  is committed, since agents load only the checked-in `.claude/settings.json`
  (`--setting-sources project`) and `.claude/skills/`, `.claude/agents/` are gitignored. So in
  bridle's repo neither the hooks nor the skills reach agents (passing layer hooks at spawn is
  34bw step 3, not built). `hooks/<event>.json`, and the "later layer wins wholesale" convention
  it and `agents/<role>.md` use, are this command's own convention; `workflow/base/hooks/PreToolUse.json`
  (arch-guard) is the one hook file, and nothing uses `agents/`. Skill sources may reference
  `{{commands.check}}`, substituted with `.bridle/config.toml`'s `[commands] check`
  (default `"just check"`, per-project — e.g. `"make check"`) so a base skill like
  `workflow/base/skills/worker/SKILL.md` doesn't hardcode one project's build tool.
- **`doctor [--repo PATH]`**: local checks on the clone (default: the current directory),
  each printed `ok`/`warn`/`FAIL` with a one-line fix (`--json`: the list). Git repo; the
  integration branch exists (the g3ck failure mode); `.bridle/config.toml` loads (the
  config loader's own error text); files it references exist (role `system_prompt`, `workflow`,
  packs, component `docs`); the machine `~/.bridle/config.toml` (a missing file is fine): each
  `[[focus]]` and `[[budget.schedule]]` block whose end is before its start without `+1d` is a
  failure naming the block and the fix; every role has a prompt (warn); `claude auth status` in doctor's own environment (FAIL, loudly, when not logged in: start the daemon from a local terminal or tmux, not over SSH, on macOS; warn if the output can't be read); `.gitignore` covers
  `.bridle/cache/`, `bridle.db` and `daemon.json` (warn); `bridle/state` exists once a
  `bridle.db` does (warn); `[ports]` range sane; git >= 2.38; `claude` on PATH; `gh` on PATH
  when `[ci] github` is on. Exits 1 if any check fails. It never fixes anything and does not
  talk to a daemon; a dry `sync` check isn't done because `sync` has no check mode.
- **`init [--repo PATH] [--name N] [--integration BRANCH] [--stack python|typescript]`**:
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
- **`systemd install`** (Linux only; ticket 4r3k): writes
  `~/.config/systemd/user/bridle-<project>.service` (`$XDG_CONFIG_HOME` honoured) for `--project`,
  or for every project `[projects]` gives `[machine] name`, and prints, but never runs,
  `systemctl --user daemon-reload`, `systemctl --user enable --now <units>` and `sudo loginctl
  enable-linger <user>` (without linger the user manager, so the daemons, waits for a login).
  Each unit runs the absolute path of the current `bridle` as `--project <name> serve --repo
  <dir>/<name> --workspace <dir>` (`<dir>` is `--projects-dir`, default the current directory's
  parent; the clone must exist), `Restart=on-failure` (a deliberate `stop-daemon` stays
  stopped), output appended to `<dir>/.bridle/daemon.log`, `PATH`/`HOME` copied from the caller,
  `WantedBy=default.target`. The port isn't in the unit: `serve` takes it from `[projects]`.
  Refuses to overwrite without `--force`.
- **`serve --detach`**: [[docs/design/agent-host/daemon#Running it|running the daemon]].
- **`tui`** is a subcommand, not a separate binary, so it shares `bridle`'s discovery,
  token and `--url`/`--project` flags like every other command. It's a thin client of
  `bridle-api`'s `Client`, with four views: an agents list (seeded from `GET
  /v1/agents`, kept live by `agent.state`/`agent.removed` events), a scrolling event
  tail (`events_stream`, which already reconnects on its own — see
  `crates/bridle-api/src/client/mod.rs`), the selected agent's transcript tail (polled
  from `Client::transcript` once a second, same model as `bridle agent logs --follow`), and
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
- **`task`** covers the `pending`/`open`/`planned`/`claimed`/`dropped`/`integrated`/`reopened` states
  (docs/design/roles-and-lifecycle.md, Task lifecycle): create (always `pending`; `task ready <id>` opens it), show, edit (title/body,
  never state), list (id/title/kind/state), plan (`open` -> `planned`), drop (a reason is
  required, recorded in the task's thread), done (`--commit` required, recorded in the
  thread; the task becomes `integrated`, which resolves its `blocks` edges and drops it from
  `queue` and `ready`) and reopen (only a dropped or integrated task can be reopened).
  `in_review` and `accepted`, and everything that depends on those, arrive with later
  tasks — see the `Planned` block below.
- **`dep add|rm`** creates or removes one coordination edge (docs/design/coordination.md).
  `bridle task dep add <task> --to <other> --kind <kind>` draws `<task> --kind--> <other>`;
  `bridle task dep add <task> --blocked-by <other>` is sugar for `--kind blocks` with `from`
  and `to` swapped (`<task>` is blocked by `<other>`, so the edge runs the other way) and
  can't be combined with `--to`/`--kind`. `dep rm` takes the same shape. Edges can't
  connect a task to itself, and a repeat of the same `(from, to, kind)` triple is a
  conflict, not a silent no-op.
- **`inbox`** has three forms:
  - Bare `bridle inbox [--all] [--mark-read]` (list, the default) shows every message to
    `me` plus every task's open question: reads `GET /v1/messages` (with `to=me`,
    `unread=true` by default) and `GET /v1/questions`. `--all` drops the `unread` filter
    (shows read messages too); `--mark-read` calls `POST /v1/messages/{id}/read` on each
    message after listing, marking every one read. It also sends `mark_read=true`, which makes
    the daemon mark what it lists read for an agent or external principal (the human's reads
    stay explicit). In JSON mode, returns both messages and
    questions; plain text prints a compact line per message/question.
  - `bridle inbox show <id> [--mark-read]` (show one message in full) fetches a single
    message to `me` by id (a question a delegate answered shows `Answered by:`; the list shows
    "answered by <who>: <first line>", visible with `--all`), prints the full header (from, kind, time, reply-to), the body,
    and the reply command (formatted as `bridle send <from> --reply-to <id> "..."`). It
    leaves the human's message unread (reading isn't handling; an agent's or external
    principal's is marked read, via `id=` and `mark_read=true`); `--mark-read` calls
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
  already open); `answer` appends an `answer` entry and clears the block. `ask --to WHO`
  (an agent, `role:NAME`, `external:NAME` or `human`; default the caller's spawner, else the
  human) also sends WHO a pointer message of kind `question`; `answer` sends the asker one of
  kind `answer`.
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
  across every daemon in the registry (`bridle daemon list`), each with its own
  discovery-resolved token, instead of just the one daemon `--url`/`--project`/cwd
  discovery would pick. `--role` is accepted but a no-op: tasks don't carry a role field
  yet (a gap, not a design decision).
- **`queue`** without a subcommand is the read-only view; `set` and `add-tier` are PM, orchestrator or human
  only and stored in the state branch's `queue.toml` (roles-and-lifecycle.md, "the queue").
  `queue set` takes one `--tier a,b` per tier, in rank order, and replaces the whole queue.
- **`budget`'s** subcommands (`hold`, `release`, `override`, `override-clear`, `max-workers`)
  are described in [[docs/design/usage-and-budget|usage and budget]]. `max-workers` sets a live
  cap that never stops running workers, only blocks new spawns and resumes.
- **`orchestrator note-session`** is the SessionStart hook `scripts/claude-orchestrator` registers for
  its own session (`bridle session orchestrator`; `scripts/claude-orchestrator` is a compat wrapper) ([[orchestrator-supervision]]). It reads the hook JSON on stdin and writes
  `$BRIDLE_HOME/orchestrator.session` as `<session id> <transcript path>` (`/clear` gives the same
  process a new id). Local, silent, never fails.
- **`focus gate`** is the `UserPromptSubmit` hook `bridle session` passes for the advisor and
  orchestrator. It does nothing on a `<task-notification>` prompt (not the human), and its quiet-hours text
  says restarting watchers is always allowed. Inside a `quiet` `[[focus]]` period ([[roles-and-config]], Focus hours) it prints
  hook JSON whose `additionalContext` is "Quiet hours (work) until 6:00 PM ET" plus the nudge
  instruction, on the first prompt of a period and again once 5 minutes have passed
  (last nudge in `$BRIDLE_HOME/focus-nudge`). With no `[[focus]]`, outside a period, or in a
  project with `focus_hours = false`, it prints nothing. In a `locked` period it blocks every prompt
  (`{"decision":"block"}`, reason "Locked until 6:00 PM. Email bridle@dev.branam.us if it
  matters."). `bridle session advisor`, `bridle session aide` and `bridle advisor start` refuse while locked. Local, never fails. An active override file (roles-and-config, Focus hours) silences it, and
  `bridle status` prints a `focus` line while one is pending or active.
- **`handover`** keeps every agent's note (orchestrator, aide, advisor/<name>, workers) as a record keyed by the writer's identity ([[orchestrator-supervision]] section 7):
  `write` reads a file or stdin (`-`), `list` shows id, time, author and first line, `show` the
  whole note. Latest wins; `bridle orchestrator prime orchestrator` prints the newest under a heading with
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
- **`bridle session advisor`** registers the session with the daemon and ends it at exit
  (best effort; see orchestrator-supervision.md, section 6). `bridle status` shows a
  `session    <identity> <project>@<machine> <tokens> up <n>m active <n>m ago` line per running
  advisor; `--json` has `sessions`. `bridle session restart <identifier> [--handover|--fresh]`
  restarts one: it SIGTERMs, then SIGKILLs, the launcher's children (by pid, never by name;
  `BRIDLE_STOP_WAIT_SECS`, default 20, each), fails without typing anything if the launcher is
  still alive, prints "restarted" only once a session with a new pid has registered, and runs
  detached (own process group, log `~/.bridle/restart-<identity>.log`) when started from inside
  the session it stops, so a session can restart itself (br-4s3z; the handover note is a
  record, so a restart never removes it); `bridle session keep <identifier>` carries on past a context warning
  (orchestrator-supervision.md, section 6). The hidden
  `bridle session note` is the advisor's SessionStart hook. `events --kind session.context`
  has the threshold crossings.
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
  here. One-time setup: if you know the project name and it is registered on this machine,
  run `bridle --project <name> token create statusline > ~/.bridle/statusline.token && chmod 600 ~/.bridle/statusline.token`
  to mint an `external:statusline` token and store it where `statusline` reads it (a fixed path under
  `$BRIDLE_HOME`/`~/.bridle`, not the workspace's own `.bridle/`, since this needs to work
  regardless of which project workspace Claude Code happens to be in). If you must use `--url`
  (for a daemon found by bare URL, not in the registry), pass `--url <daemon url> --token "$(cat <workspace>/.bridle/tokens/human)" token create statusline > ~/.bridle/statusline.token && chmod 600 ~/.bridle/statusline.token`
  instead — the `--token` is required because `--url` drops the workspace context that the human token is normally read from.
- **`arch-guard`** is Claude Code's `PreToolUse` hook (`workflow/base/hooks/PreToolUse.json`,
  matcher `Edit|Write|MultiEdit`, rendered by `bridle workflow sync`). It reads the hook JSON on stdin
  and, for an edit whose path (resolved lexically against `cwd`) is under `design/architecture/`,
  prints a `hookSpecificOutput` `permissionDecision: "deny"` unless the caller is not a worker
  agent or has claimed an `arch-revision` task; the reason tells it to run `bridle workflow arch propose`
  ([[docs/design/architecture-tier|architecture tier]]). Any error of bridle's own allows.
  Built, not wired in: it runs only where `sync` was run and the resulting `.claude/settings.json`
  committed; bridle's own repo hasn't (no `hooks` in its settings).
- **`kill-guard`** is Claude Code's `PreToolUse` hook for `Bash` (same `PreToolUse.json`, rendered
  by `sync`). It reads the hook JSON on stdin and denies a command that kills by name:
  `pkill`/`killall` as any command in a compound line, or `pgrep` together with `kill`. Plain
  `pkill *`/`killall *` are also in every role's `disallowed_tools` and the session settings' deny list; the
  hook covers what those prefix rules miss. `kill <pid>`, `kill $!` and `kill %1` are allowed.
  Rule `no-kill-by-name`; ticket 75h2.
- **`stop-check`** is Claude Code's `Stop` hook, registered only for the worker role
  ([[docs/design/coordination#How agents actually hear things (Claude Code integration)|coordination.md]],
  [[docs/spikes/05-stop-hook-findings|spike 05]]). It reads the hook's JSON on stdin; if
  `stop_hook_active` is set it allows immediately (Claude Code silently overrides a hook
  after 9 consecutive blocks in one turn, so a well-behaved hook blocks at most once per
  turn). Otherwise it lists the calling principal's own claimed tasks
  (`?claimed_by=me`) and blocks — printing the flat `{"decision":"block","reason":"..."}`
  spike 05 confirmed, not the `hookSpecificOutput` wrapper — on the first one with no
  thread entry (note, question or answer) from itself at or after `claimed_at`; naming
  the task and telling the agent to `bridle task release` it or leave a `bridle task comment`
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
  go (docs/tickets/open/one-command-orchestrator-handover-d4mz.md, step 2): the role
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
  `bridle session orchestrator` does when it uses this as `claude`'s opening prompt.
  It carries no resolved workflow rules (neither does `prime advisor`): the orchestrator and
  advisor sessions see rules only where their role file names them. Roles other than
  orchestrator, advisor, prototyper, document-reviewer, worker and planner are a clap `InvalidValue` error.
- **`prime worker|planner`** (planner = the `project-manager` rule tag) opens prime to
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
  gets the exploring agent's paragraph first ([[docs/design/explorations|explorations]]).
  Built, not wired in: no role prompt, hook or skill tells a worker or project manager to run it.
  Their resolved rules reach them anyway, in the system prompt; the facts, guides, component
  sections and the explore paragraph reach no agent.

## Planned

Commands for the phases after v1 ([[docs/proposal/build-order|build order]]),
as a first cut:

```
bridle task <cmd> at in_review|accepted          states not built yet
bridle handoff                                   bridle accept <id> (human only)
bridle inbox --inject
bridle agent spawn <role> <task>   bridle review
bridle take|give <agent>                         human takeover of a headless agent
bridle workflow rules show|propose                        `explain`/`diff --project-layer` are built (see Built)
bridle workflow trace coverage                            the other `trace` commands are built
bridle workflow explore adopt                             `new`/`conclude`/`abandon` are built
bridle usage --by project|kind|task|trend|compare   `role|model|agent` are built
```

The command name and a short alias are open:
[[command-name-and-short-alias-sqt6|command name]].
