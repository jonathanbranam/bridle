+++
id = "br-d87c"
title = "Token-less reads only from this machine (fr6q)"
kind = "bug"
state = "integrated"
created_at = "2026-09-30T10:28:46.861Z"
updated_at = "2026-09-30T10:38:11.082398Z"
size = "S"
branch = "bridle/loopback-reads"
commit = "98b14abaa46bfb556e22f5d136f40d06f4122ff5"
summary = "Token-less GET/HEAD is granted 'local' only when the TCP peer is loopback (grants_local in server.rs; missing ConnectInfo fails closed); other token-less requests 401. Daemon now serves with connect info and warns at startup when listen isn't loopback. principals.md and CLI rule 3 updated, CHANGELOG added. Test is a unit test of grants_local (no tower dev-dep for a router-level test); existing integration tests cover loopback."
+++

original id: fr6q
docs/tickets/open/token-less-reads-only-from-this-machine-fr6q.md

## Thread

### note · agent:pm-1 · 2026-09-30T10:29:20.184Z
PM brief: the ticket's 'The fix' and Tests are the spec (docs/tickets/open/token-less-reads-only-from-this-machine-fr6q.md). Files: crates/bridle-daemon/src/server.rs (auth middleware), crates/bridle-daemon/src/lib.rs (serve with into_make_service_with_connect_info::<SocketAddr>()), docs/design/agent-host/principals.md, the CLI token rule 3 in cli.md and crates/bridle (a CLI talking to a non-loopback daemon must send a token; make sure discovery/health probes such as 'bridle daemons' still work locally). Grant local only when peer ip().is_loopback(); any other token-less request is 401, reads included. Add a startup warning when listen is not loopback. Tests: token-less GET from a non-loopback peer is 401, from loopback passes; existing tests stay green (tests connect over loopback). CHANGELOG. Acceptance: just check passes. Model: Sonnet. Security fix that must land before any daemon binds to Tailscale.

### note · agent:loopback-reads · 2026-09-30T10:38:06.716Z
done: token-less reads granted local only from loopback peers, warning on non-loopback listen; just check passes (859 tests); 08eda57

### note · agent:manager-2 · 2026-09-30T10:38:11.082Z
integrated: 98b14abaa46bfb556e22f5d136f40d06f4122ff5 (branch bridle/loopback-reads)
