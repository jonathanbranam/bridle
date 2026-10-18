+++
id = "br-ewhy"
title = "Stages MVP 4: bridle spawns a designer for each unblocked stage; timer to the product manager"
kind = "feature"
state = "integrated"
created_at = "2026-10-17T21:41:29.528Z"
updated_at = "2026-10-18T05:10:21.192314Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wewhy",
]
priority = "high"
priority_at = "2026-10-17T22:04:05.788409Z"
branch = "bridle/wewhy"
commit = "8a7209a28542a4f19df64fb2456c11b826d010ee"
summary = "Stages MVP 4. New daemon module stage_watch.rs, ticked from the settle-wake loop: for each pending, not-prepared version 2 ticket in docs/tickets/open with no claimed_by line and no row claim, it spawns a `designer` (name designer-<id>-<stage>, ticket set so the settle hold applies) through AgentManager::spawn, so the budget governor, load hold, drain apply; it also waits while as many designers run as max_workers. Prompt: run `ticket next <id> --json --comments`, address comments, stage-done, commit, tell manager. Claim decision: `ticket new --claim` (still a row claim; r965 not built) and the frontmatter claimed_by are both read; either blocks a spawn, and human's signature claim blocks too. Done = stage written on the project checkout, so the designer's branch must land first (a writer whose worktree shows it written is not alerted as failed). Timeout: `[stages] timeout` (default 60m, 0 = off) or writer ended without stage-done -> thread note + one message to external:advisor/product-manager, no retry; any later edit of the ticket file (restage) lets the next tick spawn again. Writers tracked in memory; a running designer-<id>-<stage> is re-adopted after restart. stages.rs moved from bridle to bridle-daemon (re-imported in bridle) so the daemon can use it. Designer role file and config prompt widened. Docs: stages.md, daemon.md, cli.md, CHANGELOG. Tests: 3 daemon tests with the fake claude (spawn once, claimed no spawn, timeout message once) and a config test. Not exercised against a real claude."
ticket = "ewhy"
+++

docs/tickets/open/stages-mvp-4-bridle-spawns-a-designer-for-each-unblocked-sta-ewhy.md

## Thread

### note · external:advisor/tj7q · 2026-10-17T22:03:57.964Z
The human, 2026-10-17 ~5:15 PM ET, to advisor (tj7q): "Yes approved. Let's go" (shared understanding of epic tj7q, and ready steps 1-5 so the build can start today; the human wants the MVP done today to use tomorrow). Readied by human via advisor (tj7q).

### note · external:advisor/tj7q · 2026-10-17T22:04:05.788Z
priority: normal -> high

### note · external:advisor/tj7q · 2026-10-18T02:30:17.435Z
advisor (tj7q), for whoever builds this: ussh's worker reports (dd5a6f8f) that "claimed by human" is a `claimed_by` frontmatter line, because row claims need `planned` and a version 2 ticket preparing is `pending`; and that 9mqh's `ticket new --claim` likely conflicts for the same reason. This step spawns only on an unclaimed stage, so it must read the same claim that `--claim` and the human's review write. Check that `--claim` and `claimed_by` agree before relying on either.

### note · agent:wewhy · 2026-10-18T04:22:05.992Z
built: 229a1859c706cbfcee0a4a52d2e6219b274455c8 (branch bridle/wewhy)

### note · agent:wewhy · 2026-10-18T04:22:09.335Z
done: daemon spawns a designer per unblocked, unclaimed stage of a pending v2 ticket and messages external:advisor/product-manager on timeout/no stage-done; just check exit 0, 1857 tests; 229a1859c706cbfcee0a4a52d2e6219b274455c8. Claim: either a row claim or a claimed_by line blocks a spawn; a stage counts as done only when written on the project checkout, so the designer's branch must land first. Not tried with a real claude.

### note · agent:manager-2 · 2026-10-18T04:22:15.520Z
integrated: 8a7209a28542a4f19df64fb2456c11b826d010ee (branch bridle/wewhy)

### note · agent:manager-2 · 2026-10-18T04:22:36.483Z
cleanup: removed agent wewhy, branch bridle/wewhy

### note · external:advisor/product-manager · 2026-10-18T05:10:21.192Z
advisor product-manager: resolved. Verified on main (8a7209a2 = worker's 229a1859): stage_watch spawns a designer per unblocked unclaimed stage with comments, waits on signatures, one timeout/ended message to PdM, designer role widened, supervisor caps/budget. Differs from ask: designer doesn't claim; daemon tracks writer by agent name (fine). Watch item for ewk8: a stage counts written only once on the project checkout, so the designer's branch must land first; not tried with real claude.
