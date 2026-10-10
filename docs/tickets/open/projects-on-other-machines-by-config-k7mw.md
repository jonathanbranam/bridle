---
id: k7mw
title: Projects on other machines, by machine config
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: [token-less-reads-only-from-this-machine-fr6q]
see: [finding-remote-daemons-from-the-laptop-xqvg, one-machine-owns-a-project-hw6c, a-clean-handover-of-a-project-between-machines-24mj, where-the-single-orchestrator-lives-hj4g]
---

## The ask

The human, verbatim (2026-09-30, via the advisor):

> We have project ownership already right? So we know which machine owns a project. So we should
> support config (per machine, manual setup KISS) that says "meta-notes lives on nuc and listens
> on this port" and then a token config that knows about machines as well, because I will migrate
> projects between boxes and I don't want to have another manual step. I'll mint a token on both
> machines so that eg laptop has a token for meta-notes both places.

> Let's KISS - no searching, trust the config.
>
> Yes remote messages need a different name. Why not use @ symbol?

> Yes this design is good. Approved.

The use case: an orchestrator or agent on one box messages one on another, e.g. "bridle 0.4 is
out". Each machine runs its own orchestrator for its local projects.

This answers [[finding-remote-daemons-from-the-laptop-xqvg|finding remote daemons]].

## The design (approved)

**1. Machine config, written by hand, the same file on every box** (`~/.bridle/config.toml`):

```toml
[machines]
mbp = "jb-mbp"        # the host to reach it by (Tailscale MagicDNS)
nuc = "nuc"

[projects]
bridle     = { machine = "mbp", port = 7401 }
meta-notes = { machine = "nuc", port = 7402 }
```

The config is trusted: `--project meta-notes` on any machine goes to
`http://<machines.nuc>:7402`, or to this machine's registry when `machine` is this one. No
probing, no reading `owner.toml`. Moving a project means editing this line on each box (with 24mj).

**2. The daemon listens on its configured port**, on loopback and the machine's Tailscale address,
never `0.0.0.0`. A project's `[daemon] listen` still overrides. Binding off loopback needs
[[token-less-reads-only-from-this-machine-fr6q|fr6q]] first.

**3. Tokens keyed by machine** in `~/.bridle/credentials.toml`. A plain key stays "this machine",
so existing files keep working:

```toml
[advisor]             # this machine's daemons
bridle = "..."
[advisor.nuc]         # the NUC's daemons
meta-notes = "..."
```

The CLI picks the entry for the machine the config names. The human mints one token per
principal, project and machine, once; minting needs a way to print the token so it can be pasted
on the other box (`token create --project` prints none today).

**4. Remote principals are named `<name>@<machine>`**, e.g. `external:orchestrator@mbp` on the
NUC's daemon. They can send, read their own inbox and query. The daemon's own
`external:orchestrator` (its wake long poll, liveness watch, handovers) is matched by exact name,
so a visitor never takes those over. `token create` refuses `@` in a local principal's name, so the
suffix always means a visitor. `BRIDLE_AS=orchestrator` talking to another machine's daemon uses
`orchestrator@<this machine>`'s token from `[orchestrator.<machine>]`.

**5. Result:** `bridle send --project meta-notes external:orchestrator "..."` works from either
box. A reply to `external:orchestrator@mbp` is forwarded by the NUC's daemon to the visitor's
home daemon on the laptop (3haz), so the laptop's side reads it in its own inbox.

## Not in scope

- Discovering where a project lives (probing, `owner.toml` lookups).
- Daemon-to-daemon forwarding. Clients talk to the daemon that owns the recipient.
