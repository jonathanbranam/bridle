+++
id = "br-xccp"
title = "A git identity per machine, so commits show which clone made them"
kind = "chore"
state = "built"
created_at = "2026-10-09T18:08:53.503Z"
updated_at = "2026-10-21T12:09:43.523362Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:xccp",
]
priority = "low"
priority_at = "2026-10-09T18:09:00.693104Z"
branch = "bridle/xccp"
commit = "7f84c655f09b4d4a29f30da4d4c96300185ba5fa"
summary = "add-a-machine.md step 4 now sets the global git identity to 'Jonathan Branam (<machine>)' with email dev@branam.us (the 2026-10-17 decision), checks it with 'git config --show-origin --get-all user.name', and warns that an included file setting another [user] must be removed. Docs only, no CHANGELOG line per the ticket. The human still checks vesta and remus by hand."
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
