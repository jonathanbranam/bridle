+++
id = "br-ec04"
title = "Gateway 3/10: list the human's to-dos and task questions across projects"
kind = "feature"
state = "planned"
created_at = "2026-10-02T23:35:47.667Z"
updated_at = "2026-10-02T23:36:09.073386Z"
size = "M"
+++

Read docs/design/human-web-ui.md sections 1, 2 and 5 first. Goal: GET /api/v1/items (name as fits): the human's to-dos (daemon GET /v1/tasks?claimed_by=human) and open task questions to the human, from every reachable project (use the discovery from gateway 2), grouped by project, decisions first, each list high priority then oldest; unreachable projects reported alongside. Uses bridle-api; no new daemon endpoints. Gateway-owned response types (they get ts-rs in task 7, so keep them plain serde structs, no daemon types leaking). Tests: ordering, grouping, two projects merged, an unreachable one reported. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: actions, retract hiding (task 5), login. Depends on gateway 2.
