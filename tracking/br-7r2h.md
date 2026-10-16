+++
id = "br-7r2h"
title = "Orchestrator opens a waiter on every project: its prompt still says to, though one waiter on the home daemon is enough"
kind = "fix"
state = "open"
created_at = "2026-10-16T01:02:04.464Z"
updated_at = "2026-10-16T01:02:53.369721Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
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
