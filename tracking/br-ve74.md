+++
id = "br-ve74"
title = "A handover request reaches the new session after the old one already handed over"
kind = "fix"
state = "claimed"
created_at = "2026-10-10T23:26:13.887Z"
updated_at = "2026-10-12T12:35:15.993460Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wve74",
]
priority = "low"
priority_at = "2026-10-10T23:26:19.511859Z"
ticket = "ve74"
+++

docs/tickets/open/a-handover-request-reaches-the-new-session-after-the-old-one-ve74.md

## Thread

### note · external:advisor/product-manager · 2026-10-10T23:26:19.511Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-11T07:22:54.353Z
kind: bug -> fix

### note · external:aide · 2026-10-12T12:33:55.247Z
Seen again, 2026-10-12 morning, on the NUC (notes project, advisor). The human (to aide), verbatim: "Stale restarts again in NUC project advisor notes. This doesn't hurt anything but is a gap we should fix." The advisor's words, as the human pasted them: "Nothing for you to do. The listener got two leftover restart notices from the last session, sent just before I started. That restart has already happened, so I didn't restart again."

### note · agent:pm-1 · 2026-10-12T12:34:24.490Z
Brief (pm-1). Model: Sonnet. Size: S-M, one branch.

Goal: a handover/restart request (the message `bridle session restart` sends, and the "context is 200k" system note) must not be delivered to the NEW session after the old one already handed over. Seen again 2026-10-12 on the NUC (notes project, advisor): two leftover restart notices reached the new session; harmless but noisy.

Start: crates/bridle/src/session.rs (~line 587, builds and sends the request), the message/inbox read path in bridle-daemon (how a message addressed to an identity is marked read and delivered across sessions), and the br-3zhx change (a waiter's message stays unread until the same session sees it), which may keep the request unread across the restart (question 2 in the ticket).

Fix, simplest that works: when a session hands over (handover written / session restarted), mark unread restart-request and context-limit messages addressed to that identity as read (or expired), so the new session never sees them. Prefer a small rule in the handover/restart path over a new message kind or expiry system (yagni). Answer the ticket's 3 questions in a comment, including whether the 200k context notes had the same problem.

Acceptance: a test that sends a restart request, performs the handover, and checks the new session's inbox has no such message; `just check` passes. Update docs/design/agent-host/ if message delivery behaviour is described there.

Out of scope: seats/identity redesign (gtzx), other message kinds.
