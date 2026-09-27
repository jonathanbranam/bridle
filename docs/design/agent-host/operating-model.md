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
2. **Clone the project's main branch** into it.
3. **Run `bridle serve` in the clone.** Bridle reads `<repo>/.bridle/config.toml`
   if present (roles, defaults); the file is optional. Bridle needs no worktree
   for itself: it never edits code, and its state is a SQLite file under
   `<workspace>/.bridle/`. Agents get worktrees under `<workspace>/wt/<agent>`,
   on branches `bridle/<agent>`. The clone's own checkout is left to the human
   and the manager.
4. **"Start working" means starting the manager**: `bridle spawn manager`, or
   `autostart = true` on its role so it starts with the daemon. Bridle is
   mechanism: it spawns, delivers, supervises, records and (later) integrates.
   Deciding what to work on is judgement, so it's the manager's job
   ([[docs/design/roles-and-lifecycle|roles]]). Until tasks exist, "the
   project's tasks" is whatever the manager's role prompt points it at.
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

## Merging completed work

**Bridle merges its own completed work into main**; the human doesn't have to.
Progress would otherwise stop at every finished branch. The manager does it,
or the orchestrator when there's no manager, in the clone:

1. The worker brings its branch up to date: it merges the **local** `main`
   (`git merge main`, never `origin/*`) into `bridle/<agent>`, resolves any
   conflicts, runs `just check` (or the project's equivalent) and commits. A
   worker never touches `main` and never fetches or merges from a remote.
2. The merger checks that `main` is an ancestor of the branch
   (`git merge-base --is-ancestor main bridle/<agent>`), that the worker's
   worktree is clean, and that the diff does what the task asked and nothing
   else. Anything short of that goes back to the worker.
3. `git merge --no-ff bridle/<agent>`. Because the branch already contains
   `main`, this can't conflict.
4. `git push origin main`, straight after the merge, so the remote never
   lags the clone. Only the merger pushes, and only `main` and release tags;
   workers never push.

The orchestrator verifies `main` after each merge (`just check`, twice, off
load). If it's red, nothing else merges until it's green again; the fix goes
forward as a normal task.

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
wait for an orchestrator to be live.

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
  daemon from the registry. `bridle daemons` lists them. Inside a workspace,
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
