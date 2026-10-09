+++
id = "br-46fe"
title = "Reviews B: signed review records on the task (v2va slice B)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T11:47:57.275Z"
updated_at = "2026-10-09T11:04:41.265870Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
size = "M"
priority = "low"
+++

Slice B of br-91b3 (read it and the v2va ticket). Blocked by slice A. A review verdict (approve / changes-needed) is a signed thread entry on the task, written with 'bridle task review <task> --verdict ...' by a reviewer or the human; signing uses the signer's bridle token as decided (assume tokens are secure; 2bzw hardens later). Settle and document the scheme: the store keeps token hashes only, so say how the daemon verifies a signature (propose one, flag it in the done report). 'bridle task show' lists required vs signed reviews and who signed. Tests: valid signature accepted, tampered body or wrong token rejected, one reviewer cannot sign as another. Docs: principals.md, coordination.md, cli.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Not queued until the human is back.

## Thread

### note · agent:pm-1 · 2026-10-01T11:47:57.276Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-09T11:04:41.265Z
watching the task
