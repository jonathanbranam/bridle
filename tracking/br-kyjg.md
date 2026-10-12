+++
id = "br-kyjg"
title = "'bridle ticket built' fails with a bare 'error: unknown: ' on a bridle-ui ticket"
kind = "bug"
state = "pending"
created_at = "2026-10-12T23:15:42.759Z"
updated_at = "2026-10-12T23:46:30.595573Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "kyjg"
+++

docs/tickets/open/bridle-ticket-built-fails-with-a-bare-error-unknown-on-a-bri-kyjg.md

## Thread

### note · external:advisor/product-manager · 2026-10-12T23:46:30.595Z
Diagnosis (advisor product-manager, 2026-10-12, read-only):

- **Cause: version skew.** bridle-ui's daemon (127.0.0.1:7405) runs **0.6.1**; the CLI and bridle's daemon are 0.6.3. The `built` route (`POST /v1/tickets/{id}/built`) came with br-stx8 (ad59aae1), first released in **v0.6.2**. On 0.6.1 the route doesn't exist: axum answers an empty-bodied 404, the client's old-path fallback (`send_with_old_path_fallback`, crates/bridle-api/src/client/mod.rs) retries `/v1/tasks/.../built`, also unmatched, and `parse_api_error` turns the empty non-JSON body into code "unknown" with an empty message: `error: unknown: `.
- The `ticket edit --body-file` oddity is likely the same skew (a newer CLI against the 0.6.1 daemon); not confirmed.

Two parts:
1. **Ops (now, no build):** bring bridle-ui's daemon to 0.6.3 (why hasn't it self-upgraded? check its `[daemon] self_upgrade` and the machine-wide setting).
2. **Fix (this ticket):** an unmatched route (empty 404 after the fallback) says what it is: "the daemon at <url> (version X) doesn't have this command; upgrade it (bridle needs >= Y)", using `/v1/health`'s version. No bare `unknown:` ever: an empty message names the status and URL.
