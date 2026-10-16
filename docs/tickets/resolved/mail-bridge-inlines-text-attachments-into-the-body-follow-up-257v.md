---
id: br-257v
title: Mail bridge inlines text attachments into the body (follow-up to m38a)
kind: fix
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-6gkv]
related: []
resolved: 2026-10-16T10:02:58Z
---

## The ask

Follow-up to br-m38a, which landed forwards but not the human's widened scope (thread, m-13480): text/* attachments are inlined into the delivered body under "Attachment: <name>", capped around 100 KB per part with a "[truncated]" marker. Images and other binary parts stay ignored for now.
