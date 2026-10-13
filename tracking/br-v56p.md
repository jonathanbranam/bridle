+++
id = "br-v56p"
title = "Web UI login survives a reboot or gateway restart: sessions are in memory only"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T21:00:37.279Z"
updated_at = "2026-10-13T22:20:05.058478Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:v56p",
]
branch = "bridle/v56p"
commit = "01c59f38a9f73af736c9674bcc35448010d18058"
summary = "Web UI sessions are now kept in <bridle home>/gateway-sessions.json (auth.rs): keyed by sha256 of the cookie value, mode 0600, atomic temp+rename, expired entries dropped on load, corrupt/missing file = no sessions. TTL 30 days sliding; the slide is written at most hourly; login and logout write at once. Cookie Max-Age follows. router/serve take a sessions_path (None = memory only, used by tests); bridle gateway passes the home path. Unit tests cover restart, expiry, slide, logout, corrupt store, hashing and mode. Docs: human-web-ui.md section 3, CHANGELOG. Caveat: a restart can lose up to an hour of slide."
ticket = "v56p"
+++

docs/tickets/open/web-ui-login-survives-a-reboot-or-gateway-restart-sessions-a-v56p.md

## Thread

### note · agent:pm-1 · 2026-10-13T22:03:46.491Z
Brief (pm-1). Model: Sonnet. Size: S-M. The Decision above is the spec: option A, sessions on disk, 30-day sliding lifetime. Start: crates/bridle-gateway/src/auth.rs (the in-memory session map, `SESSION_TTL`, the `bridle_session` cookie) and docs/design/human-web-ui.md section 3. Design: a small store under ~/.bridle/ (a file or SQLite table, whichever the gateway code already uses for state; keep it the simplest), written when a session is made, refreshed on use (slide the expiry, and write at most about once an hour so reads don't hammer the disk), deleted on logout, expired entries dropped on load. File mode 0600, written atomically. Store a hash of the session id, not the raw value, so a leaked file cannot be replayed. A missing, empty or corrupt store means no sessions (everyone logs in once), never a crash. Cookie Max-Age follows the 30 days; keep HttpOnly and SameSite=Strict. Tests: a session survives a gateway restart (new in-process gateway on the same store), expires after 30 days, slides on use, logout removes it, corrupt store is tolerated. Docs: human-web-ui.md section 3 (new lifetime and where sessions live), CHANGELOG.md. Migration: none; the first start after upgrade has an empty store, so users log in once. Out of scope: option B, per-device session lists, changing the login method.

### note · agent:v56p · 2026-10-13T22:19:58.548Z
built: 74a78b80 (branch bridle/v56p)

### note · agent:v56p · 2026-10-13T22:19:58.580Z
done: web UI sessions on disk, 30-day sliding; just check exit 0, 1646 tests passed; 74a78b80

### note · agent:manager-2 · 2026-10-13T22:20:05.058Z
integrated: 01c59f38a9f73af736c9674bcc35448010d18058 (branch bridle/v56p)
