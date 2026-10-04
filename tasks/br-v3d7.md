+++
id = "br-v3d7"
title = "br-9j2h follow-up: stored product-manager agents and unmigrated configs keep working (read-time alias)"
kind = "bug"
state = "pending"
created_at = "2026-10-04T12:48:42.938Z"
updated_at = "2026-10-04T12:48:42.938Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
+++

Orchestrator (m-4409): br-9j2h landed locally (4656be0, NOT pushed) before this arrived. The daemon now checks role == 'project-manager' (queue gate require_pm_or_human, stop_check, planner lookup), but agents in the store keep role 'product-manager' (pm-1; track-web's PM a-lykjs), and unmigrated projects' configs still say product-manager. Fix: read-time alias: a stored/configured 'product-manager' is treated as 'project-manager' everywhere the daemon looks the role up (config role table too, until a project migrates). Update docs/design/migrations.md and roles-and-config.md (the earlier 'no alias' decision is reversed, say why), CHANGELOG. Tests: a stored product-manager agent resumes with PM permissions and the queue gate; a config with the old role name still resolves. just check must pass. Do not touch other projects.
