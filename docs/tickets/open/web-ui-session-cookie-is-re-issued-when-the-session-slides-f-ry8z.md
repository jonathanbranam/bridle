---
id: br-ry8z
title: Web UI session cookie is re-issued when the session slides (follow-up to br-v56p)
kind: fix
created: 2026-10-13
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

br-v56p stores sessions on disk with a 30-day sliding expiry, but Set-Cookie (Max-Age=30d) is sent only at login, so the browser drops the cookie 30 days after login even with daily use; docs/design/human-web-ui.md says each use pushes the expiry out. Fix: re-issue the bridle_session cookie with a fresh Max-Age when the session slides (from require_session/session in crates/bridle-gateway/src/auth.rs; at most on the same hourly save is fine). Test: a used session's response carries a refreshed Set-Cookie. Verify in a browser (rule verify-ui-in-a-browser): bridle dev site up, log in, restart the dev gateway, reload, still logged in; logout clears the cookie. Do not restart the human's live gateway. From the orchestrator, m-12053 (approved by the orchestrator as a fix inside approved br-v56p work).
