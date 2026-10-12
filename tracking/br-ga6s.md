+++
id = "br-ga6s"
title = "53gy A: one-shot take-over (serve --take-over removed), owner.toml machine identity, [projects] sync via 2wnn"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T01:04:04.376Z"
updated_at = "2026-10-12T05:44:49.844967Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
    "agent:wga6s",
]
branch = "bridle/wga6s"
commit = "e73e455ea94ee3e2cc664e31043d74a020d2d914"
summary = "`bridle project take-over` (alias group of `projects`; crates/bridle/src/take_over.rs, daemon `state_branch::take_over_project`): claims owner.toml, pushes and checks origin has it, exits; no daemon, never serves. Then sets the project's [projects] entry to this machine (port bumped past one in use here) and calls 2wnn's `sync(apply)` for every [machines] entry (no second writer). `serve --take-over` removed (hidden flag that errors naming the command); Overrides/ServeOptions take_over dropped; pre-push hook, serve refusal, OwnerConflict and docs/rule text updated. owner.toml now records [machine] name (else hostname) via `bridle_api::machines::owner_names`; an old hostname record still counts as this machine, so nothing refuses after upgrade. Seam for slice B: comment in take_over_project where the released-mark check goes. Migration: no unit or template ever carried --take-over, so none needs refreshing on dalek or the NUC. Tests: take_over_test.rs (moves owner and exits, second run no-op, port move, serve refuses naming command, flag gone), unit test for the [projects] update. Note: the global BRIDLE_PROJECT env overrides the project name in take-over (as elsewhere); tests clear it."
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

### note · agent:wga6s · 2026-10-12T05:16:07.599Z
built: d90bf010e36f34537be765a623a6c5d628c43a4c (branch bridle/wga6s)

### note · agent:wga6s · 2026-10-12T05:16:07.672Z
done: bridle project take-over (one-shot), serve --take-over removed, owner.toml = [machine] name, [projects] synced via 2wnn; just check exit 0, 1564 tests passed, on 'Merge main' parent tip (checked sha = HEAD~1 of d90bf010; the second main merge compiles clean, cargo check ok, not re-tested); d90bf010

### note · agent:manager-2 · 2026-10-12T05:44:32.671Z
integrated: e73e455ea94ee3e2cc664e31043d74a020d2d914 (branch bridle/wga6s)

### note · agent:manager-2 · 2026-10-12T05:44:49.844Z
cleanup: removed agent wga6s, branch bridle/wga6s
