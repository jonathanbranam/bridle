+++
id = "br-ty37"
title = "Gateway routes a remote project's documents through its daemon (ui-9hq8 B3; needs br-7172 and B2)"
kind = "feature"
state = "planned"
created_at = "2026-10-08T12:51:40.584Z"
updated_at = "2026-10-09T00:40:03.817344Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
priority = "high"
priority_at = "2026-10-08T12:51:40.585337Z"
summary = "The gateway's six document and spec routes now ask target_of(project): Local(repo) uses bridle-docs as before; Remote calls the owning daemon's /v1/documents, links/resolve and specs through the bridle-api client with that machine's [human.<machine>] token (documents.rs, specs.rs; actions::human_token/resolve_target made pub(crate)). Daemon 400/403/404/409/415 map back to the same DocError variants; an unreachable machine is a 503 naming it; a remote write is followed by review/add on the same daemon. API shapes unchanged, so no generated types change and the UI needs no install. Tests use a fake remote daemon (read, search, stale write, links, write then review, unreachable, no token). Gateway restart IS needed to pick this up; no UI install, no migration. CHANGELOG and docs/design/human-web-ui.md updated. Note: one daemon lifecycle test (spawn_child_orphan_is_swept_on_stop) failed once under load, passed on rerun."
+++

From bridle-ui ui-9hq8 / ui-u2df (the human: read and comment on NUC documents from the phone; reply to any task). Design: bridle-ui repo docs/design/remote-docs-and-replies.md section 3 (c52371f). B1 there is br-7172 (gateway 9/10, planned).

### B3. Gateway routes remote documents through the daemon

Goal: `repo_of` becomes `target_of(project) -> Local(PathBuf) | Remote(Client)`. The six routes
in `documents.rs` and `specs.rs` call the shared code for `Local` and the B2 client for `Remote`.
API route shapes do not change (so the generated types do not change). `add_to_review` already
uses `resolve()`, so with B1 it reaches the NUC; remote writes also go through
`PUT /v1/documents/...` and the same review call. Map a daemon 404/409/415 to the same
`DocError` variants; an unreachable NUC is 503 with the existing "reason" wording (name the
machine). Acceptance: tests with a fake remote daemon: read, search, write with stale hash,
resolve-links, write adds to review. Size: S-M. Depends on B1, B2.

## Thread

### note · external:orchestrator · 2026-10-08T12:51:40.585Z
priority: normal -> high

### note · agent:pm-1 · 2026-10-08T12:52:03.345Z
pm-1: Model Sonnet. Edges: blocked by br-7172 and br-5e4k. Acceptance: just check passes. Migration: none. Landing note says whether a gateway restart or UI install is needed (rule tc7t).

### note · agent:remotedocs · 2026-10-09T00:40:03.817Z
done: gateway routes remote documents/specs through the daemon; just check exit 0, 1345 tests passed, main merged; 118f9303. Gateway restart needed, no UI install.
