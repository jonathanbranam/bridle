+++
id = "br-4a4v"
title = "Agents commit under their own name (email dev@branam.us) and push with a per-machine PAT, not the human's identity and SSH key"
kind = "feature"
state = "integrated"
created_at = "2026-10-17T21:44:09.530Z"
updated_at = "2026-10-21T12:20:46.939517Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:a4v",
]
branch = "bridle/a4v"
commit = "6e944a54f6ba212e37da0855c5e8513e6abda1cf"
summary = "Part 1: agent_env (supervisor.rs) sets GIT_AUTHOR/COMMITTER_NAME 'bridle <agent> (<machine>)' and _EMAIL bridle@branam.us for every spawned agent; state-branch commits and the orphan-branch commit use bridle@branam.us (state commits name 'bridle (<machine>)'). Tests in supervisor.rs; docs in agents.md. Part 2 (PAT) is the human's."
ticket = "4a4v"
+++

docs/tickets/open/agents-commit-under-their-own-name-email-dev-branam-us-and-p-4a4v.md

## Thread

### note · agent:a4v · 2026-10-21T12:19:57.652Z
built: d8c71ed9 (branch bridle/a4v)

### note · agent:a4v · 2026-10-21T12:19:58.474Z
done: agents commit as bridle <agent> (<machine>) <bridle@branam.us>; just check exit 0, 1962 passed; d8c71ed9

### note · agent:manager-2 · 2026-10-21T12:20:09.571Z
integrated: 6e944a54f6ba212e37da0855c5e8513e6abda1cf (branch bridle/a4v)

### note · agent:manager-2 · 2026-10-21T12:20:27.962Z
cleanup: removed agent a4v, origin branch bridle/a4v, branch bridle/a4v

### note · external:advisor/product-manager · 2026-10-21T12:20:46.939Z
advisor (product-manager): part 1 verified on main (6e944a54 holds d8c71ed9, files identical): agents and the state branch commit as bridle <agent> (<machine>) <bridle@branam.us>. Left open for the human's by-hand steps: create bridle@branam.us in Google (and optionally add it to GitHub's emails), and part 2, each machine pushing with its own PAT. Resolve once the human confirms.
