+++
id = "br-5e4k"
title = "Daemon serves its repo's documents to the human token: /v1/documents, links/resolve, specs; shared code moved out of the gateway (ui-9hq8 B2)"
kind = "feature"
state = "integrated"
created_at = "2026-10-08T12:51:40.545Z"
updated_at = "2026-10-08T23:56:26.274802Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
priority = "high"
priority_at = "2026-10-08T12:51:40.545604Z"
branch = "bridle/daemondocs"
commit = "78b87cd39b3efe0a0340f331d6cc33603484c3d0"
summary = "Daemon serves its repo's documents to the human token (br-5e4k). New crate bridle-docs holds the pure code moved out of the gateway (read/search/write_document, resolve_links, specs load/path_for, the wire types, DocError with a status() method; no axum), used by gateway, daemon and bridle-api. Gateway keeps only routes (ApiDocError wrapper) and re-exports; behaviour unchanged, tests moved with the code. Daemon routes (server.rs, human principal only): GET /v1/documents?q=, GET/PUT /v1/documents/{*path}, POST /v1/links/resolve, GET /v1/specs; a save also adds the doc to review if it has a pending thread. Client methods (stable names for br-ty37): search_documents(q), read_document(path), write_document(path, &DocumentWrite), resolve_links(&[String]), specs(). Types re-exported from bridle_api::client. Note: a GET without a token is the synthetic local principal so gets 403 (not 401); a PUT without a token is 401. Tests: crates/bridle-daemon/tests/documents_test.rs. Docs: api.md, CHANGELOG, CLAUDE.md layout. just check green (1339 tests) before merging main (which touched only nextest config and CHANGELOG). Done in one task; split (a)/(b) was not needed."
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

### note · agent:daemondocs · 2026-10-08T23:45:59.489Z
done: daemon document routes + bridle-docs crate (parts a+b together); just check exit 0, 1339 tests, green on a8ae0202; then merged main (only nextest.toml, CHANGELOG, a doc), tip b1d06997. Client methods: search_documents, read_document, write_document, resolve_links, specs.

### note · agent:manager-2 · 2026-10-08T23:56:10.546Z
integrated: 78b87cd39b3efe0a0340f331d6cc33603484c3d0 (branch bridle/daemondocs)

### note · agent:manager-2 · 2026-10-08T23:56:26.274Z
cleanup: removed agent daemondocs, branch bridle/daemondocs
