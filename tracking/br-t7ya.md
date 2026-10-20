+++
id = "br-t7ya"
title = "Workers push their own branches to origin (no force), and landed branches are cleaned up safely"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T14:36:06.928Z"
updated_at = "2026-10-20T04:33:29.475127Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wt7ya",
]
ticket = "t7ya"
+++

docs/tickets/open/workers-push-their-own-branches-to-origin-no-force-and-lande-t7ya.md

## Thread

### note · external:aide · 2026-10-19T02:02:52.983Z
The human, 2026-10-18 ~9:50 PM ET (to aide), verbatim: "i approve that; pushing branches to remote and cleaning them up aftewards". Approved as aide proposed it: workers push their own bridle/<name> branch (no force), and bridle deletes the remote branch at the same cleanup step as the local one once the ticket has landed or been dropped. No pull requests.

### note · agent:pm-1 · 2026-10-19T02:03:37.193Z
Brief (pm-1). Right-sized: the human approved the whole ask (aide note above); remote cleanup (ask item 2) moved to its own ticket that runs after this one. THIS ticket is item 1: a worker pushes its own branch. Model: Sonnet.
Build: narrow the worker deny rule from all pushes to "its own bridle/<name> branch, no force": a worker may run `git push origin bridle/<its-name>` (or a `bridle push-branch` wrapper if a deny rule can't express it safely; prefer the wrapper, which refuses force, other refs and the integration branch, and runs from the worktree). Files: crates/bridle-daemon/src/config.rs (disallowed_tools defaults), the pre-push hook bridle installs (rule one-pusher-for-the-integration-branch must still refuse the integration branch from workers), docs/design/agent-host/operating-model.md, the worker role text in workflow/base/roles/worker.md (when to push: at least when marking built). A project that already has custom tool lists: say in the doc how it picks up the change (xebc). Tests: the wrapper/rule accepts own branch, refuses --force, +refspec, another branch, the integration branch, and a branch belonging to another agent. Acceptance: just check. Out of scope: remote cleanup (next ticket), pull requests.
Note: widening agent permissions is deliberate here (the human approved); keep it as narrow as the tests above.
