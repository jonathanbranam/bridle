# bridle

Runs and coordinates headless Claude Code agents for one project: a daemon per
workspace, a CLI, and an HTTP/SSE API that the CLI, your orchestrator agent and
(later) a TUI or GUI all share. The design is in [`docs/`](docs/README.md); what
runs today is [`docs/design/agent-host/`](docs/design/agent-host/operating-model.md).

**Status: v1 of the agent host.** You can spawn agents, message them, observe
them, interrupt, stop, resume and remove them. Tasks, workflow layers and the
integrator come later ([build order](docs/proposal/build-order.md)).

## Install

```
just install          # cargo install --path crates/bridle --locked
```

This needs a stable Rust toolchain and `claude` (Claude Code) on `PATH`,
logged in. For development, also install `just` and `cargo-nextest`
(`cargo install --locked just cargo-nextest cargo-deny`).

## Quickstart

```sh
mkdir ~/work/myproj-ws && cd ~/work/myproj-ws        # 1. a workspace per project
git clone git@github.com:me/myproj.git                # 2. the clone
cd myproj
bridle serve --detach                                  # 3. daemon (or `bridle serve` in the foreground)

bridle spawn worker --name w1 --prompt "Fix the flaky date test; commit on your branch; tell me when done."
bridle agents                                          # what's running
bridle logs w1 --follow                                # watch it work
bridle inbox                                           # messages agents sent you
bridle send w1 "Also update the changelog." [--when idle]
bridle interrupt w1                                    # end the current turn; agent stays up
bridle stop w1                                         # then: bridle resume w1 (same conversation)
bridle rm w1 [--delete-branch]
bridle events --follow                                 # everything, live, with who did it
bridle usage
bridle stop-daemon
```

This lays out the workspace as:

```
myproj-ws/
  myproj/        your clone, left to you and the manager agent
  wt/w1/         worker w1's worktree, on branch bridle/w1
  .bridle/       daemon state: db, tokens, transcripts, daemon.json, daemon.log
```

The CLI finds the daemon:
- from any directory inside the workspace;
- by name with `--project myproj` (or `BRIDLE_PROJECT`) from anywhere on the machine;
- by `--url` / `BRIDLE_URL`.

`bridle daemons` lists every project's daemon on the machine. Projects are fully
independent.

## Roles

`worker` (own worktree, Sonnet), `manager` (runs in the clone, coordinates,
spawns workers) and `orchestrator` are built in. You can override them, or add
your own, in `<repo>/.bridle/config.toml`:

```toml
[roles.worker]
model = "sonnet"
allowed_tools = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]
system_prompt = "workflow/base/roles/worker.md"

[roles.manager]
autostart = true          # an agent named "manager" starts with the daemon…
start_prompt = "Check your inbox and tell the human you're ready."   # …and gets this
system_prompt = "workflow/base/roles/manager.md"
```

Every agent can use `bridle` from its Bash tool with its own identity.
Agents message each other and you with `bridle send`.

## Your orchestrator agent

Your own agent, for example Claude Code with `claude remote-control`, drives
bridle through the CLI under its own identity:

```sh
bridle token create orchestrator            # as the human; prints a token once
export BRIDLE_TOKEN=<token>                 # in the orchestrator's environment
```

Its actions show up as `external:orchestrator` in `bridle events`, yours as
`human`, and agents' as `agent:<name>`. Inside any Claude Code session the CLI
refuses to fall back to your human token, so an agent can't act as you by
accident. This is attribution, not security, on a single-user machine.

## Remote workforce

```sh
bridle serve --detach --listen 0.0.0.0:7433     # on the remote host (no TLS: use SSH/VPN)
ssh -L 7433:localhost:7433 host                  # on the laptop
export BRIDLE_URL=http://localhost:7433 BRIDLE_TOKEN=<token>   # a URL needs an explicit token
```

Your orchestrator and a future TUI then work entirely through bridle, with no
local checkout needed.

## Development

```
just check          # fmt + clippy -D warnings + tests (what CI runs)
just test-contract  # after Claude Code updates itself: the live contract suite (Haiku, ~$0.10)
```

Tests use a fake `claude`, `crates/bridle-claude/tests/fake-claude.py`, so they
cost no tokens. Claude Code isn't pinned: when it updates, `bridle status`
shows the new version and the daemon logs a warning; run `just test-contract`,
and fix bridle forward if it fails. See [`CLAUDE.md`](CLAUDE.md) for conventions.
