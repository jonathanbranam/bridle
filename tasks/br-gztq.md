+++
id = "br-gztq"
title = "Evaluate the UI: can the human see which projects and which agents consume tokens?"
kind = "research"
state = "planned"
created_at = "2026-10-05T21:06:01.619Z"
updated_at = "2026-10-05T21:06:47.004440Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: gztq
Ticket (read first): docs/tickets/open/evaluate-the-ui-can-the-human-see-which-projects-and-which-a-gztq.md
Goal (research): evaluate the bridle UI (bridle-ui with the gateway, crates/bridle-gateway and the UI it serves) against the human's need: can they see, across ALL projects, which projects and which agents are consuming tokens, how much, over what period? Read the gateway API and the UI pages as built (and planned: br-s6cj Tasks page, br-7sd9 System page, br-xxw9 usage history), `bridle usage --json` and how per-agent tokens and cost are recorded (docs/design/agent-host/, storage.md). Do not run live tests or spend tokens; reading code and docs, and looking at a running UI/daemon read-only, is enough.
Output: write the findings into the ticket body (what the UI shows today, what is missing, what each gap needs from bridle vs the UI, cross-project aggregation given one shared account budget, xypj), and file a feature ticket (`bridle ticket new`) per gap, small and linked with `see` to gztq. Do not make tasks; the PM sizes and schedules them. Note anything that overlaps br-xxw9 rather than duplicating it.
Acceptance: findings and follow-up tickets written; `bridle ticket check` clean. No code changes.
Model: Sonnet. Out of scope: building any of it.
