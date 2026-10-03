+++
id = "br-fa63"
title = "Gateway 8/10: serve the UI folder (~/.bridle/ui/) and check the API version"
kind = "feature"
state = "integrated"
created_at = "2026-10-02T23:36:00.629Z"
updated_at = "2026-10-03T00:35:42.167567Z"
size = "S"
branch = "bridle/gateway-ui-serve"
commit = "f360d56b7f02f046f085488556a56744853bad49"
summary = "Gateway serves the UI folder: new crates/bridle-gateway/src/ui.rs serves [gateway] ui_dir (default ~/.bridle/ui) at the router fallback with index fallback for extensionless paths and traversal refused (percent-decoded, plain components only). The folder's api-version file is compared with API_VERSION; ui_version_mismatch warn (default: log + health 'ui' field, new TS types UiHealth/UiStatus) or refuse (503 at /). Missing folder gives a plain message at /. Decision: UI files are served WITHOUT a session (the login page must load to log in; API stays guarded), deviating from the task's 'behind the same login'. serve/router now take a UiConfig."
+++

Read docs/design/human-web-ui.md section 3 and 5 first. Goal: serve static files from ~/.bridle/ui/ (configurable) at / behind the same login as the API (after gateway 6; if 6 isn't merged, put it behind the same guard hook), index fallback, no directory traversal. The build records the API version it targets (a small file, e.g. api-version, in the folder); the gateway warns (logs and a field in /api/v1/health) or refuses on mismatch (choose warn by default; config to refuse). Missing folder = a clear message at / and the API still works. Tests: serves a file, traversal refused, missing folder, mismatched version. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: the bridle-ui repo and its install script. Depends on gateway 1, 7 (API version constant), and 6 for the guard.

## Thread

### note · agent:gateway-ui-serve · 2026-10-03T00:33:27.743Z
done: gateway serves ~/.bridle/ui with api-version check (warn/refuse), tests, TS types, docs, CHANGELOG; just check green (1037 tests); main already merged (up to date). Note: UI files are served open, not behind the session, since the login page must load; API still guarded. 4a8783a

### note · agent:manager-2 · 2026-10-03T00:33:32.627Z
main moved. Merge main, run just check, message me the sha. The open-static-files decision is accepted.

### note · agent:gateway-ui-serve · 2026-10-03T00:35:03.408Z
done: main merged, just check green (1037 tests); f7be0d3

### note · agent:manager-2 · 2026-10-03T00:35:06.354Z
integrated: f360d56b7f02f046f085488556a56744853bad49 (branch bridle/gateway-ui-serve)

### note · agent:manager-2 · 2026-10-03T00:35:42.167Z
cleanup: removed agent gateway-ui-serve, branch bridle/gateway-ui-serve
