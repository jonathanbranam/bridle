---
id: br-v56p
title: "Web UI login survives a reboot or gateway restart: sessions are in memory only"
kind: feature
created: 2026-10-13
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-13 (to aide, during focus time; held until 5:00 PM ET), verbatim:

> I'm pretty annoyed at having to log in to the bridle UI constantly whenever a reboot happens, which happens a lot. I would like a solution for some type of longer-lived token. I assume we're making a session token that gets lost when the system reboots or something like that. This is a pretty basic authentication thing. We're using pretty simple authentication stuff here, but I want something that survives a reboot going forwards. I'm constantly logging into that thing, and it's just a hassle.

## What's there now (aide, checked against the code)

`crates/bridle-gateway/src/auth.rs`: sessions live in an in-memory `HashMap` behind the
`bridle_session` cookie (`HttpOnly`, `SameSite=Strict`), with a 12 h TTL (`SESSION_TTL`). So the
login is lost on any gateway restart (reboot, upgrade, self-upgrade, restart), not only a reboot,
and in any case after 12 hours. Design: `docs/design/human-web-ui.md` section 3.

## Options (aide's sketch, for discussion)

- A. Keep sessions on disk (a small table or file under `~/.bridle/`), and lengthen the TTL (e.g.
  30 days, sliding). Survives restarts; logout and expiry still work.
- B. A signed, stateless cookie (HMAC key kept on disk, generated once). No session store; can't
  revoke one session, only rotate the key (logs out everywhere).
- Either way: a longer lifetime is a deliberate trade, given access is Tailscale-only and the
  gateway already treats a stolen session as able to answer and check off, not run work.
