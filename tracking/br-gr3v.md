+++
id = "br-gr3v"
title = "token pair homes the orchestrator in 'bridle' or the holder's first project, never <machine>-home: vesta's orchestrator is homed in dotfiles-local and needs a waiter per daemon"
kind = "bug"
state = "built"
created_at = "2026-10-16T01:58:15.267Z"
updated_at = "2026-10-16T03:32:46.701121Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:gr3v",
]
branch = "bridle/gr3v"
commit = "62d392e4ca0eb9817bca7c89042b6ba925f8c9aa"
summary = "token_pair.rs: home_for takes the holder machine and returns <machine>-home first for the orchestrator; entry_home also makes the orchestrator a visitor (home <machine>-home) on the holder's own non-home daemons, minted with the new hidden `pair-mint --replace-direct` (revokes the old direct token so external_exists is false and the existing wake forwarding fires). pair-check treats --home alone as visitor (no-home check). No daemon change: existing visitor-wake forwarding is reused, so ask 2 landed in the same change. Tests: home_for, mint calls, repair moves nuc's recorded home. Docs: cli.md, orchestrator-supervision.md, CHANGELOG. Migration: re-run bridle token pair on dalek, vesta, remus. Not tested against a live daemon."
ticket = "gr3v"
+++

docs/tickets/open/token-pair-homes-the-orchestrator-in-bridle-or-the-holder-s-gr3v.md

## Thread

### note · external:aide · 2026-10-16T01:58:57.968Z
From the human, via aide (~10:05 PM ET), verbatim, on br-gr3v (orchestrator homed in bridle/first project, never <machine>-home; no same-machine forwarding): "fix this; yes, ready; get this fixed". Readied. Please build it overnight.

### note · external:advisor/product-manager · 2026-10-16T01:59:11.321Z
advisor product-manager: this is the code half of br-7r2h (7r2h fixes the role prompts to one waiter on the home daemon; this makes token pair home the orchestrator in <machine>-home, and item 2 makes same-machine daemons forward to it, so one waiter really is enough). Build tonight per the human; coordinate with 7r2h's branch (prompt wording must match what this builds).

### note · agent:pm-1 · 2026-10-16T02:23:57.410Z
Brief (pm-1). Model: Sonnet. Size: M. The human readied it ("fix this ... get this fixed") and wants it built overnight; it is the code half of br-7r2h (prompts), whose wording must match what this builds. Front of the queue. Files: crates/bridle/src/token_pair.rs (`home_for` ~586, `TOKEN_ROLES` ~41, the two callers ~498 and ~688, tests ~1021), docs/design/agent-host (orchestrator-supervision.md "Wakes are also mail"), docs/context/add-a-machine.md if it describes homes, CHANGELOG.
Step 1 (ask 1): the orchestrator's home on a machine is its `<machine>-home` project when the holder's machine has one (br-8uxe); otherwise today's fallback. Do not hard-code "bridle" for the orchestrator role. Re-pairing then moves existing visitors there through br-8usa's re-mint path: verify with a test that a re-pair after the home appears changes the recorded home of `external:orchestrator@<machine>` on the other daemons (the vesta case: visitor_home=dotfiles-local must become vesta-home). Tests: with a home project -> home; without -> the old order; the existing home_for tests keep passing for other roles. Check the aide and advisor roles still get what br-qca7/br-8usa expect.
Step 2 (ask 2, decide and build if small): today each local project daemon gives the orchestrator a direct token, so it needs one waiter per local daemon. Make local daemons forward the orchestrator's mail to `<machine>-home` the way remote visitors are forwarded (use the existing visitor/forwarding mechanism; read docs/design/agent-host/orchestrator-supervision.md and how a visitor token with a home forwards), so one waiter on the home daemon is enough. Same-machine means the orchestrator holds a visitor token (home = <machine>-home) on each local daemon instead of a direct one. If this needs a new mechanism rather than reusing visitor tokens, stop, write the finding on the thread and tell the manager (the PdM will decide); land step 1 regardless.
Migration: existing orchestrator tokens are replaced when `bridle token pair` is re-run; say so in the CHANGELOG and the done note, so the human re-runs pair on dalek, vesta and remus after the upgrade. Acceptance: just check. Out of scope: the role prompts (br-7r2h), creating the home projects.

### note · agent:gr3v · 2026-10-16T03:32:46.652Z
built: 62d392e4ca0eb9817bca7c89042b6ba925f8c9aa (branch bridle/gr3v)

### note · agent:gr3v · 2026-10-16T03:32:46.701Z
done: both asks built (home_for -> <machine>-home for orchestrator; same-machine visitor token + pair-mint --replace-direct so local daemons forward wakes, existing mechanism reused). just check exit 0, 1728 tests, main already merged; 62d392e4ca0eb9817bca7c89042b6ba925f8c9aa. Migration: re-run bridle token pair on dalek, vesta, remus.
