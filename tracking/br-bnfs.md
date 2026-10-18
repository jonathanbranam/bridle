+++
id = "br-bnfs"
title = "Deep links to a human to-do open the task page and fail; open the to-do page instead, also when a to-do id is typed into the task page"
kind = "fix"
state = "integrated"
created_at = "2026-10-10T02:24:24.483Z"
updated_at = "2026-10-18T15:02:41.103710Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
    "agent:wbnfs",
]
priority = "low"
priority_at = "2026-10-10T02:24:24.517442Z"
branch = "bridle/wbnfs"
commit = "b16ce8a013216b1fbfac50357fc5f5b1cc346148"
summary = 'Bridle side of br-bnfs. `bridle link <ticket id>` now asks the daemon (best effort; a failed lookup falls back to the ticket URL) and, for a task claimed by `human`, prints `<base>/p/<project>/todos/<id>` (crates/bridle/src/link.rs). The gateway TaskSummary (so TaskDetail too) gets `todo: bool` (claimed by human and state claimed), bindings regenerated. Documented in docs/design/cli.md (bridle link). UI must do (bridle-ui, not touched): serve a `/p/<project>/todos/<id>` route that opens the to-do page scrolled to that to-do; when the ticket page loads a task with `todo: true` (including a typed to-do id), redirect there instead of showing "not found". Note: the to-do route name `todos/<id>` is my choice, since no UI doc was available locally; change it in link.rs `todo_url` if the UI uses another. Check ran green on 4e1d code (1872 passed); main merged after was docs-only.'
ticket = "bnfs"
+++

docs/tickets/open/deep-links-to-a-human-to-do-open-the-task-page-and-fail-open-bnfs.md

## Thread

### note · external:aide · 2026-10-10T02:24:24.517Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-10T02:24:37.160Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T02:24:37.195Z
advisor (product-manager): placed: theme human-ui, no epic, low (the human: "low pri"). Pending; after the machine-setup work and the comment work's turn.

### note · external:advisor/product-manager · 2026-10-11T07:22:56.821Z
kind: bug -> fix

### note · external:advisor/product-manager · 2026-10-17T15:26:11.642Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wbnfs · 2026-10-18T13:33:35.814Z
built: 665b8a8bd58a826952375e93d8e0c110a4d2e9a0 (branch bridle/wbnfs)

### note · agent:wbnfs · 2026-10-18T13:33:36.720Z
done: bridle link prints /p/<project>/todos/<id> for human to-dos; gateway task summary/detail has todo:bool; just check exit 0, 1872 passed; 665b8a8b (merge of main, docs-only; check ran on its first parent)

### note · agent:wbnfs · 2026-10-18T13:33:38.703Z
Bridle side done. UI must: add route /p/<project>/todos/<id> (to-do page scrolled to that to-do), and redirect there when the task page loads a task with todo: true (also when a to-do id is typed in), instead of 'not found'. See summary.

### note · external:advisor/product-manager · 2026-10-18T13:34:11.815Z
From advisor (product-manager): UI half submitted to bridle-ui as ui-gb7q (route /p/<project>/todos/<id> and the todo:true redirect). Resolve bnfs once it lands on main; the ask's two items finish with ui-gb7q.

### note · agent:manager-2 · 2026-10-18T15:02:41.103Z
integrated: b16ce8a013216b1fbfac50357fc5f5b1cc346148 (branch bridle/wbnfs)
