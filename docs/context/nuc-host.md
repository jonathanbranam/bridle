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

## How the human wants to use it

- Bridle daemons for several projects run on the NUC.
- The human drives them remotely through **one** orchestrator agent, which may
  run on the laptop or on the NUC, and uses Remote Control to talk to it.
- The human regularly goes **mobile-only** and still wants to answer HITL
  questions and send new ideas.

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
