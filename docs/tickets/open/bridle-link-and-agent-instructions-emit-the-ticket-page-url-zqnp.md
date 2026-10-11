---
id: br-zqnp
title: bridle link and agent instructions emit the ticket page URL /p/{project}/tickets/{id}
kind: feature
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-22ab]
---

## The ask

bridle-ui's ui-utp5 (22ab step 8, landed 9d04ff6) made the one ticket page /p/{project}/tickets/{id}, and updated docs/design/url-scheme.md (Final formats, section 5). Old URLs (/p/{p}/tasks/{id}, /task?id=, /ticket?id=) redirect, so nothing is broken; but bridle's own emitters still build the old form. Change `bridle link` (crates/bridle/src/link.rs) and the agent instructions that cite link formats (CLAUDE.md 'Link formats' and any role/rule text) to emit /p/{project}/tickets/{id} per url-scheme.md. Acceptance: `bridle link br-xxxx` prints the new form; its tests updated; just check passes. Model: Haiku. Asked for by bridle-ui's manager-2 (m-1308).
