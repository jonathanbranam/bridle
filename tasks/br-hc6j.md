+++
id = "br-hc6j"
title = "Gateway: reply to any task (POST /api/v1/projects/{project}/tasks/{id}/reply) (ui-u2df B4)"
kind = "feature"
state = "pending"
created_at = "2026-10-08T12:51:40.499Z"
updated_at = "2026-10-08T12:51:40.500181Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
priority = "high"
priority_at = "2026-10-08T12:51:40.500181Z"
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
