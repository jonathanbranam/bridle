+++
id = "br-5e4k"
title = "Daemon serves its repo's documents to the human token: /v1/documents, links/resolve, specs; shared code moved out of the gateway (ui-9hq8 B2)"
kind = "feature"
state = "planned"
created_at = "2026-10-08T12:51:40.545Z"
updated_at = "2026-10-08T12:52:02.614675Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
priority = "high"
priority_at = "2026-10-08T12:51:40.545604Z"
+++

From bridle-ui ui-9hq8 / ui-u2df (the human: read and comment on NUC documents from the phone; reply to any task). Design: bridle-ui repo docs/design/remote-docs-and-replies.md section 3 (c52371f). B1 there is br-7172 (gateway 9/10, planned).

### B2. Document endpoints on the daemon

Goal: the daemon serves its own repo's documents to a human-token caller. Routes (daemon, under
`/v1/`, human principal only; names follow the gateway's): `GET /v1/documents?q=`,
`GET /v1/documents/{*path}`, `PUT /v1/documents/{*path}`, `POST /v1/links/resolve`,
`GET /v1/specs`. Bodies and errors are the gateway's existing types (`Document`, `DocumentWrite`,
`DocumentSaved`, `DocumentMatches`, `ResolvedLinks`, `ProjectSpecs`; 404 / 400 / 415 / 409 / 403).
Files: move the pure functions of `crates/bridle-gateway/src/documents.rs` (`read_document`,
`write_document`, `search_documents`, `resolve`, `resolve_links`, hashing, the git commit on the
checked-out branch) and `specs.rs` (`load`, `path_for`) into a module both crates use (put it in
`bridle-api` or a small new crate; pick whichever avoids a gateway-to-daemon dependency), add the
routes in `crates/bridle-daemon`, and the client methods in `bridle-api/src/client/mod.rs`. The
write must keep every safety rule in the module header (plain repo-relative names, hash check,
checked-out branch only, never detached HEAD, 2 MB cap). Tests: the existing documents tests
move with the code; add daemon route tests (read, stale hash 409, `..` path 400, no token 401,
agent token refused). Acceptance: `curl` with the human token reads and edits a doc on a test
daemon. Size: M. Split if large: (a) move the code, no behaviour change; (b) daemon routes and
client.

## Thread

### note · external:orchestrator · 2026-10-08T12:51:40.545Z
priority: normal -> high

### note · agent:pm-1 · 2026-10-08T12:52:02.614Z
pm-1: Model Sonnet. Right-size: if the move of the gateway code into the shared module plus the daemon routes would pass about 200K tokens of context, do (a) the move with no behaviour change first, land it, and file (b) as its own task via 'bridle task new --from br-5e4k'. Acceptance: just check passes. Migration: none (new daemon routes; reach daemons on upgrade). br-ty37 is blocked on this task; keep the client method names stable and say them in the done note.
