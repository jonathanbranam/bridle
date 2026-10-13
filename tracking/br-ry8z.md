+++
id = "br-ry8z"
title = "Web UI session cookie is re-issued when the session slides (follow-up to br-v56p)"
kind = "fix"
state = "claimed"
created_at = "2026-10-13T22:20:53.849Z"
updated_at = "2026-10-13T22:26:27.506337Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:ry8z",
]
ticket = "ry8z"
+++

docs/tickets/open/web-ui-session-cookie-is-re-issued-when-the-session-slides-f-ry8z.md

## Thread

### note · agent:pm-1 · 2026-10-13T22:21:52.376Z
Brief (pm-1). Model: Sonnet. Size: S. The ask above is the spec (follow-up to br-v56p, integrated 01c59f38). In crates/bridle-gateway/src/auth.rs, re-issue the bridle_session cookie (same attributes: HttpOnly, SameSite=Strict, Max-Age 30 days) in the response whenever the session slides (the hourly save is the trigger; no Set-Cookie on every request). Test: a used session's response carries a refreshed Set-Cookie after the slide interval and not before. Browser check per rule verify-ui-in-a-browser on a throwaway `bridle dev site up` site only (log in, restart that site's gateway, reload, still logged in; logout clears the cookie); never restart the human's live gateway. Docs: human-web-ui.md section 3 if it states the cookie behaviour, CHANGELOG.md. Migration: none. Out of scope: any change to the stored format.
