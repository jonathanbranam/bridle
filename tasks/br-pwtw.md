+++
id = "br-pwtw"
title = "br-5paw follow-up: gateway document PUT commits on main too"
kind = "bug"
state = "open"
created_at = "2026-10-04T01:15:10.719Z"
updated_at = "2026-10-04T01:15:10.719Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
+++

Orchestrator direction (m-4206): the document write must commit on whatever branch is checked out, main included (the trial doc gtzx lives on main). Remove the main/master/dev refusal in crates/bridle-gateway/src/documents.rs; keep refusal for detached HEAD, the hash check, path-inside-project, single-file commit. Update its test, docs/design/human-web-ui.md and CHANGELOG line. just check must pass.
