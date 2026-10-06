+++
id = "br-gztq"
title = "Evaluate the UI: can the human see which projects and which agents consume tokens?"
kind = "research"
state = "planned"
created_at = "2026-10-05T21:06:01.619Z"
updated_at = "2026-10-06T21:58:28.596398Z"
created_by = "external:aide"
watchers = ["external:aide"]
summary = "Research, no code. The UI cannot yet show which projects or agents consume tokens: it shows a lifetime cost per agent and the account's window percent, per project, with no token totals, no period and no cross-project view. The daemon already serves /v1/usage, /v1/usage/breakdown (by role/model/agent, since) and /v1/usage/history; the gateway proxies none. Findings are in ticket gztq. Filed qhsa (gateway usage routes), n4p9 (Usage page), 368g (question: the human's interactive sessions are invisible). History charts stay with xxw9. bridle ticket check is clean for these tickets (other tickets' pre-existing errors remain)."
+++

original id: gztq
Ticket (read first): docs/tickets/open/evaluate-the-ui-can-the-human-see-which-projects-and-which-a-gztq.md
Goal (research): evaluate the bridle UI (bridle-ui with the gateway, crates/bridle-gateway and the UI it serves) against the human's need: can they see, across ALL projects, which projects and which agents are consuming tokens, how much, over what period? Read the gateway API and the UI pages as built (and planned: br-s6cj Tasks page, br-7sd9 System page, br-xxw9 usage history), `bridle usage --json` and how per-agent tokens and cost are recorded (docs/design/agent-host/, storage.md). Do not run live tests or spend tokens; reading code and docs, and looking at a running UI/daemon read-only, is enough.
Output: write the findings into the ticket body (what the UI shows today, what is missing, what each gap needs from bridle vs the UI, cross-project aggregation given one shared account budget, xypj), and file a feature ticket (`bridle ticket new`) per gap, small and linked with `see` to gztq. Do not make tasks; the PM sizes and schedules them. Note anything that overlaps br-xxw9 rather than duplicating it.
Acceptance: findings and follow-up tickets written; `bridle ticket check` clean. No code changes.
Model: Sonnet. Out of scope: building any of it.

## Thread

### note · agent:ui-tokens-eval · 2026-10-06T21:45:10.949Z
done: findings in ticket gztq; filed qhsa, n4p9, 368g; commit 3f7ed5c8. Messages m-5946/m-5947 (br-2y3m postmortem brief) looked meant for another agent; not acted on.

### note · agent:manager-2 · 2026-10-06T21:58:01.860Z
manager-2: land refused: main clone has uncommitted edits to the gztq ticket file (not mine; others' work). Needs the owner to commit or stash it, then I retry 'bridle task land br-gztq --branch bridle/ui-tokens-eval'. Branch is ready (3f7ed5c8, based on main, tickets only).

### note · external:aide · 2026-10-06T21:58:28.596Z
aide: the uncommitted gztq ticket edit was mine (tasks: [br-gztq] from 'bridle ticket task'); committed in f4ce99eb with stx8 and xxw9's. Please retry the land. The other uncommitted 'tasks:' edits in the main clone (95mu, ukpm, btx7, m9sd, 75zr, 3haz, 2ax5, 9z2d) aren't mine.
