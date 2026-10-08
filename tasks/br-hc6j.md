+++
id = "br-hc6j"
title = "Gateway: reply to any task (POST /api/v1/projects/{project}/tasks/{id}/reply) (ui-u2df B4)"
kind = "feature"
state = "integrated"
created_at = "2026-10-08T12:51:40.499Z"
updated_at = "2026-10-08T13:54:46.164849Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
priority = "high"
priority_at = "2026-10-08T12:51:40.500181Z"
branch = "bridle/gwreply"
commit = "d9fe8714fe93da3e61e8970f50b0602099388469"
summary = "Gateway: new Action::Reply (actions.rs) trims text, refuses empty (MissingText reply), calls note_task with the human token; works in any task state; route /projects/{project}/tasks/{id}/reply uses the existing generic action route. Daemon check: POST /v1/tasks/{id}/note calls emit_comment, which notifies claimer and watchers like an agent comment, so no daemon change. Remote projects work once br-7172 lands. Regenerated bindings/ActionRequest.ts (doc comment). CHANGELOG and human-web-ui.md updated. NEEDS a gateway restart (new route behavior); no UI install needed (no UI change, no wire type change)."
+++

From bridle-ui ui-9hq8 / ui-u2df (the human: read and comment on NUC documents from the phone; reply to any task). Design: bridle-ui repo docs/design/remote-docs-and-replies.md section 3 (c52371f). B1 there is br-7172 (gateway 9/10, planned).

### B4. Reply to any task (gateway)

Goal: `POST /api/v1/projects/{project}/tasks/{id}/reply` with the existing `ActionRequest`
(`{ text }`). Add `Action::Reply` in `actions.rs`: trims, rejects empty (`MissingText("reply")`),
calls `client.note_task(id, text)` with the human token, so the thread entry is authored `human`.
Works for any task state. Returns the existing `ActionResult` with `action: "reply"`. Check, and
fix in the daemon only if it fails, that a human note on a task notifies the claimer and watchers
the way an agent's `task comment` does. Reply only; no `comment` kind. Files: `actions.rs`,
`lib.rs` (route accepts the new action word), tests in `actions.rs`. Acceptance: a fake daemon
receives the note with the human token; empty text is refused; a remote project works once B1 is
in. Size: S. Depends on nothing for local projects; remote needs B1.

## Thread

### note · external:orchestrator · 2026-10-08T12:51:40.500Z
priority: normal -> high

### note · agent:pm-1 · 2026-10-08T12:52:01.056Z
pm-1: Model Sonnet. Acceptance: just check passes. Migration: none (gateway route; the gateway restarts onto it, so the landing note must say whether a gateway restart or UI install is needed, rule tc7t). Out of scope: a comment kind, remote projects (they work once br-7172 lands).

### note · agent:gwreply · 2026-10-08T13:54:06.587Z
done: gateway reply action; check exit 0, 1322 tests passed; 814b1173. Needs gateway restart, no UI install.

### note · agent:manager-2 · 2026-10-08T13:54:46.164Z
integrated: d9fe8714fe93da3e61e8970f50b0602099388469 (branch bridle/gwreply)
