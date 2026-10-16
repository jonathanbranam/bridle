+++
id = "br-7r2h"
title = "Orchestrator opens a waiter on every project: its prompt still says to, though one waiter on the home daemon is enough"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T01:02:04.464Z"
updated_at = "2026-10-16T04:03:52.517387Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:7r2h",
]
branch = "bridle/7r2h"
commit = "be9bf1de375ce5df03e9ed4e197a5a80360b7ae0"
summary = "The orchestrator role no longer tells a fresh orchestrator to keep one waiter per project. The startup step and the Watch duty (workflow/base/roles/orchestrator.md) now say to keep one waiter, on the home daemon, which covers every project forwarding to it. A project that does not forward is named for `bridle token pair` (human-run; it pairs visiting tokens and is safe to re-run), not given its own waiter. The aide and advisor roles (workflow/base/roles/aide.md, advisor.md) said the orchestrator watches a project only through a waiter there; they now say it watches its home daemon only. A test in crates/bridle/src/commands/orchestrator.rs guards the old per-project wording. CHANGELOG has the line under Unreleased. Not built: step 3 of the ticket (detect unforwarded projects at startup) needs daemon code and should be its own ticket. Check: just check exit 0 on d45c92f2 (main d0fad88d merged in), 1725 run, 1725 passed, 6 skipped."
ticket = "7r2h"
+++

docs/tickets/open/orchestrator-opens-a-waiter-on-every-project-its-prompt-stil-7r2h.md

## Thread

### note · external:advisor/product-manager · 2026-10-16T01:02:13.782Z
Readied on the human's go, 2026-10-15 ~9:02 PM ET: "Yes, let's file that as a ticket and fix it."

### note · agent:pm-1 · 2026-10-16T01:02:53.369Z
Brief (pm-1). Model: Sonnet. The human readied it and wants it fixed ("exactly what I'm seeing"). Land in this order and stop at the first boundary that gets long, telling the manager (follow-ups need the PdM to ready them).
Step 1 (docs, the heart of it, small): rewrite workflow/base/roles/orchestrator.md so the startup step (~39) and the Watch duty (~85) say: ONE waiter, on the home daemon (consistent with ~99-104); delete "each project you hold a token for has a waiter" as a duty; name what to do for a project that does not forward to the home (step 2's setup command), not "open another waiter". Then `git grep -n "waiter" workflow/ docs/design/agent-host/orchestrator-supervision.md` and `bridle prime orchestrator`'s source for the same leftover in other roles (aide, advisor) and rules, and fix them. Also update the project's own copy if one exists under .bridle/ (check). Keep ASCII. CHANGELOG. This also touches the orchestrator.md that br-qca7/br-37hz edited (both landed or queued; rebase on main).
Step 2 (the one-command setup): find how a visitor token for `external:orchestrator@<machine>` is set up today (docs/design/agent-host/orchestrator-supervision.md, `bridle token pair`, br-3haz/rhba); if a single existing command does it, name it in the prompt text; if none exists, stop and say so to the manager rather than build one.
Step 3 (detect, only if small): at startup the orchestrator (or `bridle status --json`) lists projects on this machine that neither are its home nor forward to it. Prefer an existing status field; if it needs new daemon code, stop and tell the manager, since it becomes its own ticket.
Acceptance: just check (docs changes may be checked by the prompt tests; update those if the text is pinned). Verify by reading the final startup and Watch text: exactly one waiter is instructed. Out of scope: removing `--all-projects` (br-kuvh, unblocked by this), the dalek-home project.

### note · agent:7r2h · 2026-10-16T01:16:59.497Z
Handoff (7r2h): docs and test committed on bridle/7r2h at a285b73c (role wording in orchestrator.md, aide.md, advisor.md; test in commands/orchestrator.rs; CHANGELOG under Unreleased). main is already merged in. Step 3 (detect unforwarded projects) not built: needs daemon code, to be its own ticket. Still open: the just check run (.bridle/check.log, .bridle/check.exit) is waiting on the build lock; its exit status, test count, summary and built marking come next, then the sha goes to manager-2.

### note · agent:7r2h · 2026-10-16T01:46:58.341Z
Update (7r2h): the first just check on a285b73c was killed mid-nextest by the background time limit (1603/1724 tests not run; fmt and clippy had passed), so it proves nothing. Merged main (d0fad88d, br-54sp) into the branch at d45c92f2 and restarted just check with the longest background limit. Result, sha and built marking come next.

### note · external:advisor/product-manager · 2026-10-16T01:59:11.472Z
advisor product-manager: related br-gr3v (readied by the human tonight): token pair homes the orchestrator in bridle/first project, never <machine>-home, and local daemons don't forward; that is why vesta's orchestrator still needs a waiter per daemon. 7r2h's prompt and gr3v's code must agree.

### note · agent:7r2h · 2026-10-16T02:10:50.574Z
done: orchestrator role says one waiter on the home daemon, not one per project (aide and advisor roles, a regression test, CHANGELOG line); just check exit 0 on d45c92f2 (1725 run, 1725 passed, 6 skipped); summary is on the ticket. NOT marked built: 'bridle ticket built' was refused with 'forbidden: worker agents cannot use agent lifecycle endpoints', so please mark it built and land with --checked-commit d45c92f2. Step 3 (detect unforwarded projects) not built, needs its own ticket.

### note · agent:manager-2 · 2026-10-16T02:23:48.280Z
built: d45c92f2 (branch bridle/7r2h)

### note · agent:7r2h · 2026-10-16T02:23:52.710Z
7r2h: PdM says token pair homes in the first project and local daemons do not forward, so 7r2h should land with gr3v (commit d45c92f2 unchanged). Waiting for the PdM's call on landing order.

### note · agent:manager-2 · 2026-10-16T02:23:55.911Z
manager-2: holding the land of d45c92f2 until br-gr3v (the code half) lands, so the prompt is true when it ships. Not landing it in the current chain.

### note · agent:7r2h · 2026-10-16T02:23:56.330Z
Worker (7r2h): read the PdM comment on gr3v. Until gr3v lands, the prompt is wrong for vesta: token pair homes the orchestrator in the first project, not vesta-home, and local daemons do not forward, so a project on vesta still needs its own waiter. As built, the role text says to keep one waiter on the home and to name a non-forwarding project for token pair. Recommendation: land br-7r2h together with gr3v, so the prompt and the code agree. If gr3v is far off, add an interim line for vesta (its own waiter per daemon until forwarding is live). Not changing the prompt or landing order without the PdM's call. Built commit is d45c92f2.

### note · agent:7r2h · 2026-10-16T02:23:58.672Z
7r2h: agreed the prompt and gr3v must agree. manager-2 is holding the land of d45c92f2 until gr3v lands, so the text ships with its code. No change made on my side; the ticket comment has the detail.

### note · external:advisor/product-manager · 2026-10-16T02:24:10.905Z
advisor product-manager (PdM call): agreed, land 7r2h together with or right after gr3v, not before.

### note · agent:7r2h · 2026-10-16T02:24:12.740Z
7r2h: the PdM confirmed the call: land 7r2h with or right after gr3v, not before. Built commit d45c92f2 unchanged; nothing more from me unless gr3v needs a prompt change.

### note · agent:7r2h · 2026-10-16T03:59:39.433Z
done: br-gr3v has landed and main is merged into bridle/7r2h; just check exit 0 on 285d75c5 (1734 run, 1734 passed, 6 skipped). Land with --checked-commit 285d75c5 (the earlier d45c92f2 is superseded by the merge).

### note · agent:manager-2 · 2026-10-16T04:03:26.724Z
integrated: be9bf1de375ce5df03e9ed4e197a5a80360b7ae0 (branch bridle/7r2h)

### note · agent:manager-2 · 2026-10-16T04:03:52.517Z
cleanup: removed agent 7r2h, branch bridle/7r2h
