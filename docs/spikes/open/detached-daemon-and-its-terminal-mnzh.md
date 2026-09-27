---
id: mnzh
title: "Spike: does a detached daemon survive its terminal closing?"
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

`bridle serve --detach` re-executes itself in a new **process group**
(`process_group(0)`), not a new session. The workspace forbids `unsafe`, so
there's no `setsid` via `pre_exec`. The daemon ignores SIGHUP.
`docs/agent-host.md` §6.3 @ c192bfc said:

> `bridle serve --detach` re-executes itself in a new session with output
> going to `.bridle/daemon.log`.

That isn't what was built, and [[docs/design/agent-host/daemon#Running it|the daemon doc]]
now says so. Verify by hand:

- start `bridle serve --detach` from a terminal, then close the terminal
  window (and try the same over SSH, and inside tmux);
- check whether the daemon and its agents stay up, and whether agents get
  SIGHUP or SIGTTOU.

## Why it matters

The operating model is "start bridle in the background and walk away",
including on a remote host over SSH. If closing the terminal kills it, the
options are `setsid(1)` / `nohup` in the re-exec, a small safe wrapper crate
(`nix::unistd::setsid` in the child is fine once it runs as its own process),
or a launchd or systemd unit.

## Notes

- The v1 CLI e2e test covers `--detach` + `daemons`, but not terminal
  hang-up.
