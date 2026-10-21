+++
id = "br-xccp"
title = "A git identity per machine, so commits show which clone made them"
kind = "chore"
state = "built"
created_at = "2026-10-09T18:08:53.503Z"
updated_at = "2026-10-21T12:42:35.337464Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:xccp",
]
priority = "low"
priority_at = "2026-10-09T18:09:00.693104Z"
branch = "bridle/xccp"
commit = "7f84c655f09b4d4a29f30da4d4c96300185ba5fa"
summary = "add-a-machine.md step 4 now sets the global git identity to 'Jonathan Branam (<machine>)' with email dev@branam.us (the 2026-10-17 decision), checks it with 'git config --show-origin --get-all user.name', and warns that an included file setting another [user] must be removed. Docs only; the CHANGELOG Internal line was added because land requires one. The human still checks vesta and remus by hand."
ticket = "xccp"
+++

docs/tickets/open/a-git-identity-per-machine-so-commits-show-which-clone-made-xccp.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T18:09:00.693Z
priority: normal -> low

### note · agent:xccp · 2026-10-21T12:09:41.307Z
built: 7f84c655f09b4d4a29f30da4d4c96300185ba5fa (branch bridle/xccp)

### note · agent:xccp · 2026-10-21T12:09:43.523Z
done: add-a-machine.md step 4 sets global user.name 'Jonathan Branam (<machine>)' and email dev@branam.us, with the show-origin check; docs only, no CHANGELOG; just check exit 0, 1960 run, 1960 passed, 6 skipped; 7f84c655f09b4d4a29f30da4d4c96300185ba5fa

### note · agent:manager-2 · 2026-10-21T12:09:58.785Z
Land refused: a CHANGELOG line is required. Add one under ## Unreleased / ### Internal (top of section) about the git identity step in add-a-machine.md, run just check, commit, message me. Also make sure the ticket has a summary (bridle ticket summary).

### note · agent:xccp · 2026-10-21T12:30:15.175Z
done: CHANGELOG Internal line added (top of section, br-xccp); summary written. just check exit 0 on the new tip, 1960 run, 1960 passed, 6 skipped. Tip is b62bd5186ed7358bc2e728b52b1dc1558ba87cd8 on bridle/xccp, pushed. Note: the first check run had one failure, restart_hold_test cancel_ends_an_upgrade_drain_quietly (timed out waiting for the drain to lift, under load). It passes alone (7/7); the rerun of just check was green. Not re-marked built: the ticket is already built at 7f84c655 and bridle refuses a new claim (not ready to claim), so 'ticket built' on b62bd518 was refused. Please land with --checked-commit b62bd518 or tell me how you want the built mark moved.

### note · agent:xccp · 2026-10-21T12:42:35.337Z
done: merged main (6e944a54, br-4a4v) into bridle/xccp, no conflicts; just check exit 0 on the merged tip, 1962 run, 1962 passed, 6 skipped; tip 2461fd288621567d7594a1b704ae40d4ec5a0530, pushed. The ticket stays built at 7f84c655 (not re-marked: claim refused). Land with --checked-commit 2461fd28.
