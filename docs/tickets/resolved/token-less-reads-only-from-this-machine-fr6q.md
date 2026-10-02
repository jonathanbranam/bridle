---
id: fr6q
title: Token-less reads only from this machine
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [finding-remote-daemons-from-the-laptop-xqvg]
closed: 2026-10-02T00:43:36.199360Z
---

## The ask

The human, verbatim (2026-09-30, via the advisor): "yes, let's add that as a task - reads only
allowed from local; is that enforceable?"

## The problem

A request with no bearer token that is a `GET` or `HEAD` is let in as the `local` principal
(`crates/bridle-daemon/src/server.rs`, the auth middleware; the comment there says "The daemon
only listens on 127.0.0.1"). That is safe only while the daemon listens on loopback. `[daemon]
listen` can be set to any address, and the NUC plan ([[docs/context/nuc-host|NUC host]],
[[finding-remote-daemons-from-the-laptop-xqvg|finding remote daemons]]) wants a daemon bound to
its Tailscale address. Then anything on the tailnet or LAN could read agents, messages, tasks,
logs and events without a token.

## The fix (sketch)

Enforceable: the peer address of a TCP connection is set by the handshake and can't be spoofed
by the client.

- Serve with `into_make_service_with_connect_info::<SocketAddr>()` (today `axum::serve(listener,
  app)` in `crates/bridle-daemon/src/lib.rs`), and in the middleware grant `local` only when the
  peer `ip().is_loopback()`. Any other token-less request gets 401, reads included.
- Anything forwarded to the daemon from this machine (an `ssh -L` tunnel, a local reverse proxy)
  arrives from loopback and still counts as local. That's acceptable: reaching it took a login on
  the machine.
- Maybe a startup warning when `listen` isn't loopback, naming that tokens are now required for
  every request.
- Update [[docs/design/agent-host/principals|principals]] ("Read access without a token") and
  the CLI's token rule 3, which relies on the daemon's tolerance.

Tests: a token-less `GET` from a non-loopback peer is 401; from loopback it still passes.

## Resolution

Resolved by: br-d87c (98b14ab), br-dabd (cbaf6c0)
