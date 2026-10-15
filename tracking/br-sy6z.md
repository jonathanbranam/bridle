+++
id = "br-sy6z"
title = "take-over on the machine that released a project says 'already owned by this machine': say it is released and how to take it back"
kind = "fix"
state = "integrated"
created_at = "2026-10-15T00:24:52.488Z"
updated_at = "2026-10-15T23:54:47.259193Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:sy6z",
]
branch = "bridle/sy6z"
commit = "76432aef25bda2326967e16c4bbf020df80cc92c"
summary = 'take-over on the machine that released a project now prints "released by this machine (host) at <time>; it is not owned now. To take it back, run `bridle project start` here; to move it, run take-over on the new machine." New TakeOverOutcome::ReleasedByThisMachine in state_branch.rs; "already owned" kept for the unreleased case. No behaviour change. Runbook (docs/context/move-a-project.md) did not use the old wording, so no edit. Test added in take_over_test.rs.'
ticket = "sy6z"
+++

docs/tickets/open/take-over-on-the-machine-that-released-a-project-says-alread-sy6z.md

## Thread

### note · external:advisor/product-manager · 2026-10-15T21:56:05.619Z
Readied by the human via advisor product-manager, 2026-10-15 ~5:20 PM ET: "Yes, ready both".

### note · external:advisor/product-manager · 2026-10-15T21:56:19.400Z
Correction: readied ~5:55 PM ET, not ~5:20.

### note · agent:pm-1 · 2026-10-15T21:56:50.006Z
Brief (pm-1). Model: Haiku. Size: S. Wording only, per the ticket body. Files: crates/bridle/src/take_over.rs (~62, the printed message), crates/bridle-daemon/src/state_branch.rs (~1087, TakeOverOutcome::AlreadyOwner: add the released case, e.g. a variant or a field carrying the released time, so the CLI can word it; no behaviour change, the take-over result stays as it is). Message for the released case: "<p>: released by this machine (<host>) at <time>; it is not owned now. To take it back, run `bridle project start` here; to move it, run take-over on the new machine." Keep "already owned by this machine" for the unreleased case. Update the move-a-project runbook back-out lines if they use the old wording (git grep -n "already owned" docs/ crates/). Tests: both cases print the right text. CHANGELOG line. Acceptance: just check. Out of scope: any change in take-over behaviour. No migration.

### note · agent:sy6z · 2026-10-15T23:31:36.385Z
done: take-over on a released-by-this-machine project now says released + how to take it back; just check exit 0, 1718 tests passed, ran on tree equal to commit 08110b41 (CHANGELOG line added while check ran; changelog only); main already merged

### note · agent:manager-2 · 2026-10-15T23:31:42.878Z
built: 08110b41 (branch bridle/sy6z)

### note · agent:manager-2 · 2026-10-15T23:54:44.418Z
integrated: 76432aef25bda2326967e16c4bbf020df80cc92c (branch bridle/sy6z)

### note · agent:manager-2 · 2026-10-15T23:54:47.259Z
cleanup: removed nothing; kept agent sy6z (background job pid 89427)
