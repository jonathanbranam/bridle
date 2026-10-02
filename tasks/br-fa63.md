+++
id = "br-fa63"
title = "Gateway 8/10: serve the UI folder (~/.bridle/ui/) and check the API version"
kind = "feature"
state = "planned"
created_at = "2026-10-02T23:36:00.629Z"
updated_at = "2026-10-02T23:36:10.741715Z"
size = "S"
+++

Read docs/design/human-web-ui.md section 3 and 5 first. Goal: serve static files from ~/.bridle/ui/ (configurable) at / behind the same login as the API (after gateway 6; if 6 isn't merged, put it behind the same guard hook), index fallback, no directory traversal. The build records the API version it targets (a small file, e.g. api-version, in the folder); the gateway warns (logs and a field in /api/v1/health) or refuses on mismatch (choose warn by default; config to refuse). Missing folder = a clear message at / and the API still works. Tests: serves a file, traversal refused, missing folder, mismatched version. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: the bridle-ui repo and its install script. Depends on gateway 1, 7 (API version constant), and 6 for the guard.
