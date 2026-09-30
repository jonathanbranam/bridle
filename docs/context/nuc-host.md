# The workforce host: an Intel NUC

*Recorded 2026-09-27. The machine bridle is meant to run on first. The full
setup guide is the human's
[NUC Server Rebuild artifact](https://claude.ai/artifact/1Csq9NwiFu4NcuDN4bMBG9).*

## The machine

| | |
|---|---|
| Hardware | Intel NUC5i5RYH: i5-5250U (2 cores / 4 threads), 16 GB RAM |
| Storage | new 500 GB M.2 **SATA** SSD for the system; the old 2 TB HDD stays as `/srv` for data and backups |
| OS | Ubuntu Server 26.04 LTS, boots to a console; Xfce only on demand |
| Updates | unattended-upgrades for all stable updates, **automatic reboot at 04:00 when needed**, Livepatch |
| Tools | Node 24 (NodeSource), `gh`, tmux, Claude Code (native installer) |

The human expects it to be slow with many agents and doesn't mind a slow swarm.

## How it's reached

- **Tailscale** is the main way in, with no router ports open. MagicDNS gives it
  the name `nuc`. `ufw` denies inbound except SSH from the LAN and anything on
  `tailscale0`.
- **SSH** is plain OpenSSH with keys only, over Tailscale (`ssh nuc`). A
  reverse tunnel through an EC2 instance is the fallback if Tailscale is down.
- **Remote Control:** `claude remote-control --name nuc` runs at boot in a tmux
  session under a systemd user unit (`claude-rc`), with linger enabled so user
  services start without a login.

So a bridle daemon on the NUC is reachable from the laptop by
`ssh -L <port>:localhost:<port> nuc`, or directly on its Tailscale address if
it listens there. Either way the traffic is encrypted.

With `[machine] name = "nuc"` and `[projects] <name> = { machine = "nuc", port = N }` in
`~/.bridle/config.toml`, `bridle serve` listens on 127.0.0.1:N and the NUC's Tailscale address
(from `tailscale ip -4`), never `0.0.0.0`. Off loopback every request needs a token, reads included.

## How the human wants to use it

- Bridle daemons for several projects run on the NUC.
- The human drives them remotely through **one** orchestrator agent (possibly
  per project, via `bridle session orchestrator --project <name>`), which may
  run on the laptop or on the NUC, and uses Remote Control to talk to it.
- The human regularly goes **mobile-only** and still wants to answer HITL
  questions and send new ideas.

## Running sessions for specific projects

`bridle session orchestrator` and `bridle session advisor` take a project name and run from any
directory (the old `scripts/claude-*` are wrappers around them):

```bash
bridle session orchestrator --project <name>    # e.g., --project meta-notes
bridle session advisor --project <name> [alias] # e.g., --project bridle alice
```

The project defaults to the `BRIDLE_PROJECT` environment variable, or `bridle` if unset.
The project is included in the session name (e.g., `orch-meta-notes-nuc`, `advisor-alice-bridle-nuc`
with suffix `nuc`) and passed to bridle for daemon discovery and token selection from
`~/.bridle/credentials.toml`.

## How bridle is run for now

By hand. The human, 2026-09-27: "I don't think bridle is stable enough as a
daemon unless it can auto update itself. I'll run and manage it myself for
now." So no systemd or launchd unit, and no unattended deployment on the NUC,
until bridle can update itself. That makes the reboot and detached-daemon
tickets below less urgent.

## What this changes in the design

Each of these is an open ticket:

- [[where-the-single-orchestrator-lives-hj4g|Where the single orchestrator lives]]
- [[answering-hitl-questions-from-mobile-u6wk|Answering HITL questions from mobile]]
- [[finding-remote-daemons-from-the-laptop-xqvg|Finding remote daemons from the laptop]]
- [[agents-and-nightly-auto-reboots-t39j|Agents and nightly automatic reboots]]
- [[process-containment-on-linux-2mj9|Process containment on Linux]] (spike)
- [[remote-control-for-a-hosted-orchestrator-bagg|Remote Control for a hosted orchestrator]] (spike)
- Machine-level capacity, filed under
  [[how-project-daemons-share-one-budget-xypj|how project daemons share one budget]]

## Moving a project to another machine

A project has one serving machine ([[docs/design/storage#The state branch|storage]], "Ownership").
To move one (old machine to new):

1. **Old machine, push the work.** Push the integration branch and any working branches you want to
   keep (`git push origin main <branch>...`). Uncommitted worktree changes don't move.
2. **Old machine, stop the daemon** with `bridle stop-daemon`. One writer only: don't start the new
   daemon first. Shutdown stops the agents, flushes tasks and pushes `bridle/state` (bounded to
   10 s); check that `git push origin bridle/state` says up to date, or run it yourself.
3. **New machine, clone** the repo and build or install `bridle`. `bridle/state` comes with it from
   origin; no manual `git fetch origin bridle/state:bridle/state`.
4. **New machine, `bridle serve --take-over`** in the clone (add `--detach` to background it).
   Without `--take-over` it refuses, naming the old host, because `owner.toml` on `bridle/state`
   still names it. It also refuses, naming both SHAs, if origin is unreachable, lacks
   `bridle/state`, or `bridle/state` or the integration branch has diverged from origin's, or if
   the integration branch needs a fast-forward and its checkout is dirty; fix that by hand
   (nothing is rebased or forced), then rerun. The claim is pushed as a normal state flush.
5. **Create tokens**: `bridle token create <name>` for each principal (e.g. `orchestrator`,
   `advisor`, `human`); with the project known they're saved in `~/.bridle/credentials.toml`
   rather than printed. Tokens don't move with the project.
6. **Start the sessions** with the project-aware commands, as in "Running sessions for specific
   projects" above: `bridle session orchestrator --project <name>` and
   `bridle session advisor --project <name> [alias]`.
7. **Old machine, mark its clone tools-only** (below) so nobody starts a second daemon there.

Messages (the inbox) live in SQLite, not on the state branch, and do not move; read anything
still needed on the old machine first. Tasks, edges, open questions and claims come from the state
branch (`bridle rebuild` if the new machine's database looks stale). Cross-machine message sync is
not built. Ticket hw6c.

## Tools-only clones

A clone kept only for its tools (not the project's home) is listed in `~/.bridle/config.toml`:
`[machine] tools_only = ["/home/<user>/src/bridle"]`. Then run `bridle machine tools-only-install`
in it once. `bridle serve` and the orchestrator and advisor scripts refuse there; the git hooks
refuse commits and pushes (`--no-verify` is the deliberate override). Ticket hw6c.
