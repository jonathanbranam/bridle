+++
id = "br-kyjg"
title = "'bridle ticket built' fails with a bare 'error: unknown: ' on a bridle-ui ticket"
kind = "bug"
state = "integrated"
created_at = "2026-10-12T23:15:42.759Z"
updated_at = "2026-10-17T19:30:37.004719Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/wkyjg"
commit = "ed905f18c0a3f7f61bb0851733c5b7da69d5104b"
summary = "send_json/send_unit in bridle-api client: an empty 404 after the old-path fallback now errors 'the daemon at <url> (version X) doesn't have this command; upgrade it' (version via a direct /v1/health fetch, omitted if it fails); parse_api_error's empty non-JSON body now says 'empty reply, HTTP <status>' instead of a bare 'unknown: '. Two tests; CHANGELOG line. No design doc documents the wording. The unconfirmed 'ticket edit --body-file' oddity was skipped."
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

### note · external:orchestrator · 2026-10-12T23:47:56.056Z
Ops part done (orchestrator, 2026-10-12 23:47Z): bridle-ui, data-contracts and track-web daemons on dalek were all on 0.6.1, because only bridle's own .bridle/config.toml sets self_upgrade = true, so the other daemons never restart onto the shared installed binary. Restarted all three with 'bridle daemon restart --project <p>'; each now reports 0.6.3 and resumed its agents. The ticket's fix (a named error for an unmatched route) stands.

### note · external:advisor/product-manager · 2026-10-17T15:26:08.857Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wkyjg · 2026-10-17T18:55:26.246Z
done: unmatched-route error names daemon+version, empty errors name HTTP status; just check exit 0, 1826 tests passed; 9c41c541

### note · agent:manager-2 · 2026-10-17T19:30:22.949Z
integrated: ed905f18c0a3f7f61bb0851733c5b7da69d5104b (branch bridle/wkyjg)

### note · agent:manager-2 · 2026-10-17T19:30:30.099Z
manager-2: land failed (git commit of the squash failed, empty error), and bridle/wkyjg does not contain main (CHANGELOG Unreleased has moved). wkyjg: merge main into your branch (resolve CHANGELOG, keep every line), rerun just check, commit, message me with the new sha.

### note · agent:manager-2 · 2026-10-17T19:30:37.004Z
cleanup: removed agent wkyjg, branch bridle/wkyjg
