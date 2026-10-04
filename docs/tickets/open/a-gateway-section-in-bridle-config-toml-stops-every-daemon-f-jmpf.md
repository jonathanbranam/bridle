---
id: jmpf
title: A [gateway] section in ~/.bridle/config.toml stops every daemon from starting (unknown field)
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [anmx, essy]
tasks: []
---

## The ask


Found 2026-10-04 09:10 ET, setting up the gateway on dalek. The human followed
`docs/design/human-web-ui.md` and added the login to `~/.bridle/config.toml`:

```toml
[gateway]
username      = "..."
password_hash = "..."
bind          = "100.100.189.100:7878"
```

`bridle gateway` started fine. Then bridle-ui's daemon, freshly moved to launchd, exited 1 at once,
over and over (`daemon.log`):

```
17 | [gateway]
   |  ^^^^^^^
unknown field `gateway`, expected one of `machines`, `projects`, `daemon`, `roles`, ...
```

The daemon's config (`crates/bridle-daemon/src/config.rs`, `#[serde(deny_unknown_fields)]`)
reads the same machine file and doesn't know `gateway` (not on main at 1abe575 either). Every
daemon on the machine fails to start or restart while the section is there; the ones already
running are fine only until their next restart. bridle's self-upgrade was due to restart it about
5 minutes later, and the orchestrator confirmed there's no way to hold a self-upgrade and the
rollback re-execs the old binary on the same config, so bridle would have stayed down. The human
commented the section out at 09:11; the running gateway kept its loaded config.

`human-web-ui.md` section 6 says the section is "new **optional machine config** ... a machine that
doesn't run a gateway never sees it", and `crates/bridle/src/gateway.rs` says "a bad `[gateway]`
section can't affect any other command". Neither held for the daemon. Until fixed, the gateway
can't be restarted (or installed as a service, br-c657) without breaking every daemon.

## Wanted

- The daemon's machine config accepts `[gateway]` and ignores it (the gateway alone parses it).
  A test: a machine `config.toml` with a full `[gateway]` section loads in the daemon.
- `bridle daemon doctor` passes with it.
- Worth checking: any other section only another command reads (`[mail]` is listed; anything
  else) has the same trap.

Priority: critical (the orchestrator marks it ready).
