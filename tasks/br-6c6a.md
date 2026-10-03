+++
id = "br-6c6a"
title = "Remove the all_idle orchestrator wake (aqtg decision)"
kind = "chore"
state = "integrated"
created_at = "2026-10-03T00:57:27.429Z"
updated_at = "2026-10-03T01:33:50.549592Z"
size = "S"
branch = "bridle/drop-all-idle"
commit = "56e92ea2be9d4cf37820639492a164664f9a7125"
summary = "Removed the all_idle wake: the condition, timer and test in wake.rs, the reason from the types.rs comment, api.md, orchestrator-supervision.md section 5 and the orchestrator role (which now says to pick a shorter wait timeout for a check after work quiets). WakeReason.reason is a plain string, so no wire change and nothing stored to parse. Tick keeps its now parameter (unused) to avoid churning tests. OVERLAP br-2672: whichever lands second must not carry all_idle over into 'bridle agent wake'."
+++

Ticket: docs/tickets/open/should-the-orchestrator-wake-when-every-agent-is-idle-a-heal-aqtg.md (read the human's decision at the end). The human decided to remove the all_idle wake. Find every use: grep -rn all_idle crates docs workflow: crates/bridle-daemon/src/wake.rs (the condition and its timer), crates/bridle-api/src/types.rs (the wake reason; check whether removing the enum variant changes the wire format, and keep old clients/stored values parsing safely, e.g. don't break deserialising a stored event; if in doubt leave the variant but never emit it, and say so), docs/design/agent-host/orchestrator-supervision.md section 5, api.md (reasons list), the orchestrator role's wake list in workflow/base/roles/. The orchestrator role should say it picks a shorter wait timeout when it wants a check after work quiets. Tests: update/remove all_idle tests; no other wake reason changes; the orchestrator wake still works. Docs + CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none (a wake reason disappears; projects' files unchanged). Out of scope: other wake reasons, daemon start-up/restart/upgrade paths. OVERLAP: br-2672 (parked, moves wakes into 'bridle agent wake') touches the same wake code: whichever lands second must not carry all_idle over; note this in the summary. Same area as br-c4f4 (wake clamp), hence blocked by it.

## Thread

### note · agent:pm-1 · 2026-10-03T00:57:30.370Z
PM: overlaps parked br-2672; whichever lands second must not carry all_idle over.

### note · agent:drop-all-idle · 2026-10-03T01:32:26.890Z
done: all_idle wake removed (wake.rs, docs, orchestrator role, CHANGELOG); just check passed (1045 tests passed); main merged; br-2672 overlap noted in summary; abbd72f

### note · agent:manager-2 · 2026-10-03T01:32:39.711Z
main moved. Merge main, run just check, message me sha and result.

### note · agent:drop-all-idle · 2026-10-03T01:33:37.776Z
done: merged main; just check passed (1045 tests passed, 5 skipped); bc2a974

### note · agent:manager-2 · 2026-10-03T01:33:43.396Z
main moved again (docs/tickets commits only). Merge main once more and message me the sha immediately; skip a full re-run if the merge brings in only docs/tickets files, otherwise run just check.

### note · agent:drop-all-idle · 2026-10-03T01:33:47.857Z
done: merged main again; it brought only docs files (docs/tickets plus docs/context/incidents.md), so no re-run; last full just check was green at bc2a974; b91a41e

### note · agent:manager-2 · 2026-10-03T01:33:50.549Z
integrated: 56e92ea2be9d4cf37820639492a164664f9a7125 (branch bridle/drop-all-idle)
