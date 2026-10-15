+++
id = "br-m38a"
title = "Mail bridge drops the forwarded part of a forwarded email"
kind = "bug"
state = "pending"
created_at = "2026-10-15T13:52:07.944Z"
updated_at = "2026-10-15T13:52:12.335657Z"
created_by = "external:orchestrator@vesta"
watchers = ["external:orchestrator@vesta"]
ticket = "m38a"
+++

docs/tickets/open/mail-bridge-drops-the-forwarded-part-of-a-forwarded-email-m38a.md

submitted by external:orchestrator@vesta

Reported by notes' aide on vesta (m-0044 to external:orchestrator, 2026-10-15 13:51 UTC).

What happened: an email forwarded to the notes project (Subject "Fwd: Cancel 10/22 session") arrived as notes message m-0041 (from external:mail to external:aide). The body ends at "Begin forwarded message:". The forwarded part (its headers and text) never reached external:aide.

Impact: the recipient can't act on a forwarded email, which is the common way the human hands an email to an agent.

Ask: carry forwarded content through to the message body, whether it is inline (quoted below a 'Begin forwarded message:' / '---------- Forwarded message ---------' separator) or attached as a message/rfc822 part.

Verify: forward an email (Apple Mail inline forward, and a forward-as-attachment) to a project's mail address; the delivered message body contains the forwarded headers and text.

Cause: unknown (likely the bridge keeps only the first text/plain part, or stops at the forwarded-message boundary).

## Thread

### note · external:orchestrator@vesta · 2026-10-15T13:52:07.945Z
submitted by external:orchestrator@vesta
