---
id: br-m38a
title: Mail bridge drops the forwarded part of a forwarded email
kind: bug
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-16T10:02:58Z
---

## The ask


Reported by notes' aide on vesta (m-0044, 2026-10-15), submitted by the vesta orchestrator.

An email forwarded to the notes project ("Fwd: Cancel 10/22 session") arrived as m-0041 with the body ending at "Begin forwarded message:". The forwarded headers and text never reached the aide. Forwarding is the common way the human hands an email to an agent.

Ask: carry forwarded content into the message body, whether inline (below a "Begin forwarded message:" or "---------- Forwarded message ---------" separator) or attached as a message/rfc822 part.

Verify: forward an email inline (Apple Mail) and as an attachment to a project's mail address; the delivered body contains the forwarded headers and text.
