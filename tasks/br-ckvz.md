+++
id = "br-ckvz"
title = "Tasks split from another inherit its watchers, across projects too"
kind = "feature"
state = "planned"
created_at = "2026-10-04T21:28:54.526Z"
updated_at = "2026-10-04T22:08:45.365212Z"
created_by = "external:advisor/doc-review"
watchers = ["external:advisor/doc-review"]
ticket = "ckvz"
+++

Same-project part of docs/tickets/open/tasks-split-from-another-inherit-its-watchers-across-project-ckvz.md (read it, and the xxxq watchers design and code). Build: (1) a real link from a new task to its parent: bridle task new --from <task> (name per cli.md conventions), stored on the task, and a note on the parent's thread naming the child; (2) a task made with --from inherits the parent's watchers (the creator is still added); (3) the orchestrator and manager role prompts (workflow/base/roles/) tell them to use --from when splitting a task. Runs after br-bdrc (both touch task creation and watchers; dependency edge). Existing tasks have no link: the field is optional, no migration. Acceptance: just check passes; tests for the link, inheritance and the parent-thread note. Model: Sonnet. Deferred, not dropped: part 3 of the ticket (a watcher on another project's daemon reaching its waiter): that is cy2v (--all-projects) or 3haz (cross-daemon mail) and needs the human's pick; note on the thread when you finish what the cross-project gap still is.

## Thread

### note · external:advisor/doc-review · 2026-10-04T21:48:31.897Z
From the human, via advisor (doc-review), 2026-10-04: approved for work ("Yes", in answer to whether to open it).
