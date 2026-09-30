---
id: xqvg
title: Finding remote daemons from the laptop
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [hj4g]
---

## The question

How does a client on the laptop find and reach the project daemons running on
another machine?

## Why it matters

[[docs/design/agent-host/operating-model#Several projects at once|Several projects]] selects daemons with `bridle --project <name>` from a
machine-local registry, `~/.bridle/daemons/`. Each daemon listens on
`127.0.0.1:0` by default, so its port is random. From the laptop, the registry
isn't visible and SSH forwards can't be set up ahead of time.

## Notes

Raised on 2026-09-27 while checking the NUC plan ([[docs/context/nuc-host|NUC host]]) against the agent-host design.

- **Fixed port per project**, set in `.bridle/config.toml` `[daemon] listen`.
  Forward each one with `ssh -L`, or bind to the host's Tailscale address.
  With Tailscale, the traffic is already encrypted, and the NUC's firewall
  only admits the tailnet and LAN SSH. Tokens then become the real
  authentication ([[docs/design/agent-host/principals|principals]]).
- **Run the CLI remotely:** `ssh nuc 'BRIDLE_TOKEN=… bridle --project X …'`.
  The token must be explicit. The CLI's rule 2 ([[docs/design/agent-host/principals#How the CLI picks a token|token choice]]) falls back to the human
  token when `$CLAUDECODE` is unset, and it is unset in the remote shell, so an
  orchestrator would silently act as the human. SSH doesn't forward
  environment variables by default.
- If the orchestrator lives on the NUC ([[where-the-single-orchestrator-lives-hj4g|where it lives]]), most of this
  goes away.

## Answer (2026-09-30)

Approved by the human: [[projects-on-other-machines-by-config-k7mw|projects on other machines, by
machine config]]. A hand-written machine config names each project's machine and port; tokens are
keyed by machine; remote principals are `<name>@<machine>`. Resolve this ticket when k7mw lands.
