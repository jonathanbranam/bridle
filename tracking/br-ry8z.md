+++
id = "br-ry8z"
title = "Web UI session cookie is re-issued when the session slides (follow-up to br-v56p)"
kind = "fix"
state = "integrated"
created_at = "2026-10-13T22:20:53.849Z"
updated_at = "2026-10-13T22:46:37.979881Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:ry8z",
]
branch = "bridle/ry8z"
commit = "ce415e795094f7c0d65f156af2284f770c278907"
summary = "auth.rs: Sessions::touch now reports whether the slide was saved (the hourly save); require_session and session then append a Set-Cookie with a fresh 30-day Max-Age (same attributes as login) to that response only. Test added for the hourly trigger. human-web-ui.md and CHANGELOG updated. Browser (throwaway site, API only; the site had no UI process): login, gateway restart, reload still 200, logout 204 then session 401."
ticket = "ry8z"
+++

docs/tickets/open/web-ui-session-cookie-is-re-issued-when-the-session-slides-f-ry8z.md

## Thread

### note · agent:pm-1 · 2026-10-13T22:21:52.376Z
Brief (pm-1). Model: Sonnet. Size: S. The ask above is the spec (follow-up to br-v56p, integrated 01c59f38). In crates/bridle-gateway/src/auth.rs, re-issue the bridle_session cookie (same attributes: HttpOnly, SameSite=Strict, Max-Age 30 days) in the response whenever the session slides (the hourly save is the trigger; no Set-Cookie on every request). Test: a used session's response carries a refreshed Set-Cookie after the slide interval and not before. Browser check per rule verify-ui-in-a-browser on a throwaway `bridle dev site up` site only (log in, restart that site's gateway, reload, still logged in; logout clears the cookie); never restart the human's live gateway. Docs: human-web-ui.md section 3 if it states the cookie behaviour, CHANGELOG.md. Migration: none. Out of scope: any change to the stored format.

### note · agent:ry8z · 2026-10-13T22:46:31.850Z
built: 1642d9dc (branch bridle/ry8z)

### note · agent:ry8z · 2026-10-13T22:46:31.894Z
done: session cookie re-issued with fresh Max-Age on the hourly slide; just check exit 0, 1647 tests passed (last full: 1638); checked sha 1642d9dc (main merged after the check; it brought only a ticket doc). Browser on throwaway site (API only, no UI process in the site): login ok, session 200; killed and restarted that site's gateway, reload, session still 200; logout 204 then session 401. Site down, nothing left running.

### note · agent:manager-2 · 2026-10-13T22:46:37.979Z
integrated: ce415e795094f7c0d65f156af2284f770c278907 (branch bridle/ry8z)
