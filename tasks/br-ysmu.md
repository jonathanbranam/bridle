+++
id = "br-ysmu"
title = "CI watch missed 13 red runs on main; first ci_failed wake came 40 minutes late"
kind = "bug"
state = "planned"
created_at = "2026-10-05T03:21:20.785Z"
updated_at = "2026-10-10T22:00:37.563079Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
summary = """Cause: the CI watcher followed only the newest tip of main (git ls-remote every third tick, then polled that one sha). Pushes landing while it polled, or between tip checks, were never looked at, so 13 red commits produced one ci.completed. Fix (crates/bridle-daemon/src/ci.rs): every tick lists the branch's 50 most recent runs (gh run list --branch), groups by commit, and reports each commit whose runs have all finished, once, oldest first, so the first red one wakes the manager. First look after start treats only the newest commit as news. Gh trait: remote_tip replaced by branch_runs; Run gained head_sha. New `bridle ci` (prints sha/conclusion/age/url, --json prints CiStatus, exit 0 only on success) reads the daemon's in-memory last result via status; empty until a run finishes after a daemon restart. Manager role text gets a "never merge on red" bullet. Docs: operating-model.md CI watcher, cli.md, CHANGELOG. Tests: fake CI source with several commits between polls (all recorded, first failure wakes), plus first-look and report-once tests. Caveat: a commit whose workflows start at different times could be reported when only the first workflow has finished (same as before)."""
ticket = "ysmu"
+++

Ticket: docs/tickets/open/ci-watch-missed-13-red-runs-on-main-first-ci-failed-wake-cam-ysmu.md (read it). Goal: (1) find why the CI watcher missed runs on a newly added project (watcher start? only the newest run polled? a run finishing between polls dropped?) and fix the cause so there is one ci.completed event per run on main and a wake on the first failure; (2) a small bridle command the manager can run to read the latest CI result on main (a read of what the watcher already records) so never-merge-on-red does not need gh. Files: the CI watcher in crates/bridle-daemon (grep ci.completed, ci_failed), docs/design/ (CI watcher doc; check docs/README.md index), cli.md, manager role text in workflow/base/roles for the new command, CHANGELOG. Acceptance: just check passes; a test with a fake CI source returning several runs between polls: all recorded, first failure wakes. Migration: the command and role text reach projects via bridle workflow sync / daemon upgrade; no project files change. Model: Sonnet. Out of scope: other CI providers, retry/backoff tuning.

## Thread

### note · external:orchestrator@nuc · 2026-10-05T03:21:20.786Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T03:21:29.947Z
Triage (pm-1): accept, high value: merges went in on red on an onboarded project. Ticket minted (uncommitted; needs committing on main). Stays pending: approve with `bridle task ready br-ysmu` and I plan it (Sonnet, bridle-daemon CI watcher; the manager's read command may split off as a small second task).

### note · external:advisor/product-manager · 2026-10-09T11:04:38.565Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T02:59:22.486Z
advisor (product-manager): readied. The human, 2026-10-09 ~10:55 PM ET: "if the machine work finishes up, let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters; I think the scheduled message work is also an important epic to finish up soon". New epic messaging (theme agents-and-cli), ranked right after machine-setup. Order: br-2msq, br-n7cg, br-rhba, br-ysmu, br-cufw.

### note · agent:wysmu · 2026-10-10T22:00:37.563Z
done: CI watcher now reports every commit on main (first red wakes), new 'bridle ci'; just check exit 0, 1465 tests; 92bd2e99 (main merged after the check; it only added ticket docs)
