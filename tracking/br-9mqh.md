+++
id = "br-9mqh"
title = "Stages MVP 1: ticket version 2, the stages schema, the proposal section and the stage folder"
kind = "feature"
state = "integrated"
created_at = "2026-10-17T21:41:27.823Z"
updated_at = "2026-10-18T01:50:41.388973Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:w9mqh",
]
priority = "high"
priority_at = "2026-10-17T22:04:05.691568Z"
branch = "bridle/w9mqh"
commit = "12d1085831d48601278ca5641454a794c5c33d3a"
summary = "Version 2 tickets: new crates/bridle/src/stages.rs (schema parse/render/validate, OpenSpec requires graph, proposal finder, stage states). ticket new --version 2 [--stages --review --claim] writes version: 2, stages: block and proposal template; check validates stages for v2 only; resolve moves the stage folder; show prints stage states; frontmatter reader skips indented lines (a stage named specs must not read as the specs field); daemon ticket_links reads version. Signature is provisionally a signed: line in the stage file (proposal: inside its section); step 3 may change it. docs/design/stages.md, cli.md, storage.md, CHANGELOG updated. v1 tickets round-trip unchanged (tested)."
ticket = "9mqh"
+++

docs/tickets/open/stages-mvp-1-ticket-version-2-the-stages-schema-the-proposal-9mqh.md

## Thread

### note · external:advisor/tj7q · 2026-10-17T22:03:55.210Z
The human, 2026-10-17 ~5:15 PM ET, to advisor (tj7q): "Yes approved. Let's go" (shared understanding of epic tj7q, and ready steps 1-5 so the build can start today; the human wants the MVP done today to use tomorrow). Readied by human via advisor (tj7q).

### note · external:advisor/tj7q · 2026-10-17T22:04:05.691Z
priority: normal -> high

### note · agent:w9mqh · 2026-10-18T01:17:22.517Z
built: 06658309 (branch bridle/w9mqh)

### note · agent:w9mqh · 2026-10-18T01:17:22.685Z
done: stages MVP 1 built; just check exit 0, 1841 tests passed; 06658309. Caveat: signature recorded provisionally as a 'signed:' line (step 3 may change). 'ticket show' stage states and --claim not exercised against a live daemon (unit-tested only).

### note · agent:manager-2 · 2026-10-18T01:50:26.951Z
integrated: 12d1085831d48601278ca5641454a794c5c33d3a (branch bridle/w9mqh)

### note · agent:manager-2 · 2026-10-18T01:50:41.388Z
cleanup: removed agent w9mqh, branch bridle/w9mqh
