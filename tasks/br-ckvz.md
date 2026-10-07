+++
id = "br-ckvz"
title = "Tasks split from another inherit its watchers, across projects too"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T21:28:54.526Z"
updated_at = "2026-10-07T06:36:16.481047Z"
created_by = "external:advisor/doc-review"
watchers = ["external:advisor/doc-review"]
branch = "bridle/ckvz"
commit = "9273e339eb6d22d3528af0fc1ace536e800816d1"
summary = "Added `bridle task new --from <task>`. New optional `parent` field on Task and NewTaskRequest (types.rs), stored as a `parent` frontmatter line (state_branch.rs; no SQLite column, no migration). The daemon's new_task handler refuses an unknown parent, then TaskManager::split_from sets the link, copies the parent's watchers into the child (creator stays) and adds a note naming the child to the parent's thread. Docs: cli.md, storage.md, CHANGELOG; orchestrator and manager role prompts say to use --from when splitting. Test: tasks::tests::split_from_links_inherits_watchers_and_notes_the_parent (link, inheritance, parent note, survives reload, unknown parent). Cross-project gap (part 3 of the ticket) is still open: --from only resolves a parent on the same daemon, and a watcher on another project's daemon doesn't reach a waiter on the bridle daemon; that needs cy2v (--all-projects) or 3haz (cross-daemon mail), the human's pick. Note the parent-thread note does not itself notify watchers."
ticket = "ckvz"
+++

Same-project part of docs/tickets/open/tasks-split-from-another-inherit-its-watchers-across-project-ckvz.md (read it, and the xxxq watchers design and code). Build: (1) a real link from a new task to its parent: bridle task new --from <task> (name per cli.md conventions), stored on the task, and a note on the parent's thread naming the child; (2) a task made with --from inherits the parent's watchers (the creator is still added); (3) the orchestrator and manager role prompts (workflow/base/roles/) tell them to use --from when splitting a task. Runs after br-bdrc (both touch task creation and watchers; dependency edge). Existing tasks have no link: the field is optional, no migration. Acceptance: just check passes; tests for the link, inheritance and the parent-thread note. Model: Sonnet. Deferred, not dropped: part 3 of the ticket (a watcher on another project's daemon reaching its waiter): that is cy2v (--all-projects) or 3haz (cross-daemon mail) and needs the human's pick; note on the thread when you finish what the cross-project gap still is.

## Thread

### note · external:advisor/doc-review · 2026-10-04T21:48:31.897Z
From the human, via advisor (doc-review), 2026-10-04: approved for work ("Yes", in answer to whether to open it).

### note · agent:ckvz · 2026-10-07T06:35:43.372Z
done: task new --from (parent link, inherited watchers, parent-thread note), docs and role prompts; just check exit 0, 1296 tests passed; 021a16fd (main already merged)

### note · agent:ckvz · 2026-10-07T06:35:45.509Z
Cross-project gap remains: --from only works on the same daemon; watching a task on another daemon (bridle-ui) still doesn't wake a waiter on the bridle daemon. That is cy2v or 3haz, needs the human's pick.

### note · agent:manager-2 · 2026-10-07T06:36:06.720Z
integrated: 9273e339eb6d22d3528af0fc1ace536e800816d1 (branch bridle/ckvz)

### note · agent:manager-2 · 2026-10-07T06:36:16.481Z
cleanup: removed agent ckvz, branch bridle/ckvz
