+++
id = "br-ga6s"
title = "53gy A: one-shot take-over (serve --take-over removed), owner.toml machine identity, [projects] sync via 2wnn"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T01:04:04.376Z"
updated_at = "2026-10-12T03:56:13.693557Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
    "agent:wga6s",
]
ticket = "ga6s"
parent = "br-53gy"
+++

docs/tickets/open/53gy-a-one-shot-take-over-serve-take-over-removed-owner-toml-ga6s.md

Slice of br-53gy (pm-1's split, agreed by advisor product-manager 2026-10-11 ~9:10 PM ET). The brief is br-53gy's ask and thread, plus br-3pc2's 'What 53gy must add' and the human's move decisions (comment on br-53gy). Order A -> B -> C, behind br-2wnn.

## Thread

### note · agent:pm-1 · 2026-10-12T01:04:36.961Z
Build brief (pm-1). Model: Sonnet. Read br-53gy (ask, my brief comments, the PdM's and human's comments) and br-3pc2 "What 53gy must add"; this slice is part A of A -> B -> C. Blocked by br-2wnn.

Do: (1) one-shot take-over command (`bridle project take-over`, the neighbour in crates/bridle/src/cli.rs): commits owner.toml to this machine, confirms origin has it, exits; never serves, works with no daemon running. (2) `bridle serve --take-over` removed with a clear error naming the new command; update every message, doc, rule text and the pre-push hook message that names it; move owner_push_test.rs and restart_test.rs cases. (3) owner.toml machine identity per br-3pc2. (4) Before exit, update [projects] (machine, port if it must change) on this machine and every machine in [machines] by calling 2wnn's sync code, no second writer. Take-over does not require the released mark yet (slice B adds it); leave a clearly named seam for it.
Docs: cli.md, daemon.md (ownership), add-a-machine guide, CHANGELOG. Acceptance: just check passes; tests: owner moves and exits, second run no-op, serve as non-owner refuses naming the command, [projects] sync called.
Migration: units on dalek and the NUC that run `serve --take-over` (if any): say in the done note how they are refreshed.
Out of scope: stop/start, released mark, re-pair, schedules, bridle.db (all later slices or decided out).
