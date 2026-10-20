+++
id = "br-pm2r"
title = "Delete a landed worker's remote bridle/<name> branch at the manager's cleanup step (from t7ya, part 2)"
kind = "feature"
state = "integrated"
created_at = "2026-10-19T02:03:33.360Z"
updated_at = "2026-10-20T06:18:12.171068Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wpm2r",
]
branch = "bridle/wpm2r"
commit = "57a6b3c0a45a1cdb17e6975a1db3e4edfcdc097c"
summary = "clean_up_landed_branch (server.rs) now calls worktree::delete_remote_branch before the local delete: only bridle/*, only when is_merged, only if on origin; plain 'git push origin --delete'; errors logged (warn), never fail cleanup. Test with a bare origin covers landed/unmerged/non-bridle/main/already-gone. Docs: operating-model.md; CHANGELOG."
ticket = "pm2r"
+++

docs/tickets/open/delete-a-landed-worker-s-remote-bridle-name-branch-at-the-ma-pm2r.md

## Thread

### note · external:orchestrator · 2026-10-19T02:04:12.544Z
Readied by orchestrator on the human's approval of br-t7ya (both parts), via aide 2026-10-18 ~9:50 PM ET: "i approve that; pushing branches to remote and cleaning them up aftewards".

### note · agent:wpm2r · 2026-10-20T06:17:40.623Z
built: 3b30bc1c9290ec0ffe50a33700f5386cd0a37094 (branch bridle/wpm2r)

### note · agent:wpm2r · 2026-10-20T06:17:41.083Z
done: remote branch delete at landing cleanup; just check exit 0, 1940 tests; 3b30bc1c9290ec0ffe50a33700f5386cd0a37094

### note · agent:manager-2 · 2026-10-20T06:17:57.656Z
integrated: 57a6b3c0a45a1cdb17e6975a1db3e4edfcdc097c (branch bridle/wpm2r)

### note · agent:manager-2 · 2026-10-20T06:18:12.171Z
cleanup: removed agent wpm2r, branch bridle/wpm2r
