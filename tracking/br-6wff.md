+++
id = "br-6wff"
title = "Scratch files in a project's clone are not ignored: .bridle/tmp, reports and review lists show as untracked"
kind = "chore"
state = "integrated"
created_at = "2026-10-22T01:00:17.935Z"
updated_at = "2026-10-22T04:17:39.832618Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:w6ff",
]
branch = "bridle/w6ff"
commit = "00ec252c0a38f454283374c9843f8d136d6eb52d"
summary = "Gitignore .bridle/tmp/ (agent scratch): bridle's own .gitignore, init scaffold, and new migration 0008-gitignore-bridle-tmp (shares a gitignore_line helper with 0007); advisor and orchestrator roles say scratch goes under .bridle/tmp/. Tests: 0008 appends once; init's existing gitignore test covers the new line. The two stray files (overnight-load-report, review-documents.txt) are not in my worktree and not on main; they are untracked in the main clone, left for the orchestrator to delete or keep."
ticket = "6wff"
+++

docs/tickets/open/scratch-files-in-a-project-s-clone-are-not-ignored-bridle-tm-6wff.md

## Thread

### note · external:orchestrator · 2026-10-22T01:00:42.996Z
orchestrator: committed and readied (bug/chore, rule ready-without-asking). Split from br-6gdp by pm-1.

### note · agent:w6ff · 2026-10-22T04:17:16.181Z
built: 025f40e0 (branch bridle/w6ff)

### note · agent:w6ff · 2026-10-22T04:17:16.228Z
done: .bridle/tmp/ ignored (own .gitignore, init, migration 0008), roles updated; stray files not in my worktree, left for orchestrator; just check exit 0, 1966 tests; 025f40e0

### note · agent:manager-2 · 2026-10-22T04:17:23.123Z
integrated: 00ec252c0a38f454283374c9843f8d136d6eb52d (branch bridle/w6ff)

### note · agent:manager-2 · 2026-10-22T04:17:39.832Z
cleanup: removed agent w6ff, origin branch bridle/w6ff, branch bridle/w6ff
