---
id: br-ux3h
title: "Waiter: the upgrade exit (code 6) says when (UTC), from/to version and expected downtime"
kind: feature
created: 2026-10-17
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-16 ~7:25 PM ET, to advisor (product-manager): "Do you get a message or a
specific return from the waiter before an upgrade? I think that would be useful to get a message
with a UTC timestamp or a waiter return specifically for an uograde".

Today a waiter (`bridle agent wake`) ending for an upgrade exits 6 with one stderr line, set at
shutdown in `crates/bridle-daemon/src/server.rs` (`set_stop_reason`):

```
error: daemon restarting for an internal upgrade to 0454f3b8b (requested by system); re-arm once it is back
```

It has no time, no "from" version and no hint of how long the restart takes, so a waiting agent
can't tell when it happened or when to expect the daemon back.

1. **The exit-6 line says when and what.** Add the UTC timestamp (RFC 3339), the version or
   commit it upgrades from and to, and, when known, how long the last restart took ("back in
   ~Ns"). Keep exit code 6 and the "re-arm once it is back" wording: roles key on them.
2. **Also as JSON.** With `--json`, the same fields as an object (`reason: "upgrade"`, `at`,
   `from`, `to`, `expected_back_secs`), so a script doesn't parse the text.
3. Docs: the waiter's exit codes in docs/design/cli.md (and daemon.md where the stop reason is
   described).

Done when: a waiter ended by an upgrade prints the UTC time, from/to and (when known) the expected
downtime, in text and in `--json`; exit code stays 6; tests cover both forms.

Out of scope: advance notice before the drain starts (a separate ticket if wanted).
