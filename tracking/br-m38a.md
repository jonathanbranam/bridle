+++
id = "br-m38a"
title = "Mail bridge drops the forwarded part of a forwarded email"
kind = "bug"
state = "open"
created_at = "2026-10-15T13:52:07.944Z"
updated_at = "2026-10-15T13:52:42.189839Z"
created_by = "external:orchestrator@vesta"
watchers = ["external:orchestrator@vesta"]
ticket = "m38a"
+++

Reported by notes' aide on vesta (m-0044, 2026-10-15), submitted by the vesta orchestrator.

An email forwarded to the notes project ("Fwd: Cancel 10/22 session") arrived as m-0041 with the body ending at "Begin forwarded message:". The forwarded headers and text never reached the aide. Forwarding is the common way the human hands an email to an agent.

Ask: carry forwarded content into the message body, whether inline (below a "Begin forwarded message:" or "---------- Forwarded message ---------" separator) or attached as a message/rfc822 part.

Verify: forward an email inline (Apple Mail) and as an attachment to a project's mail address; the delivered body contains the forwarded headers and text.

## Thread

### note · external:orchestrator@vesta · 2026-10-15T13:52:07.945Z
submitted by external:orchestrator@vesta

### note · agent:pm-1 · 2026-10-15T13:52:19.402Z
Brief (pm-1). Model: Sonnet. Size: S/M. File: crates/bridle-mail/src/parse.rs ~198, `msg.body_text(0)` (mail-parser): it returns only the first text part, so a forward-as-attachment (message/rfc822 part) is dropped, and an inline forward may be dropped if the client sends multipart with the quoted part as a separate text part or as HTML-only. Reproduce first with fixture .eml files in the crate's tests (inline forward as one text/plain; multipart/alternative with HTML forward; message/rfc822 attachment) and see which lose text. Fix: append the text of every text/plain part (fall back to HTML-to-text when no plain part, if the parser offers it) and, for each attached message/rfc822, a block with its From/Date/Subject/To headers and its body text, under a plain "---------- Forwarded message ----------" separator. Do not duplicate text already in the body. Keep ASCII in docs. Docs: the mail bridge design doc (find via docs/README.md) and CHANGELOG. Acceptance: tests for the three fixtures; just check. Out of scope: non-message attachments, HTML rendering fidelity. No config or migration change. Do not touch the digest or stale-binary code (br-tcae).
