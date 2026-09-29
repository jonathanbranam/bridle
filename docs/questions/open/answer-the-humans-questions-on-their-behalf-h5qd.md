---
id: h5qd
title: The orchestrator answers a question to the human, and it closes for the human
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [kp3f, a7h3]
---

## What happened

pm-1 asked the human a question (m-1210, track-web: where should the manager merge?) that a
standing rule already answered. The orchestrator answered pm-1 directly (m-1211), but it
can't close a message addressed to someone else (`POST /v1/messages/m-1210/read` returns
`forbidden: not the recipient`), so m-1210 stayed open in the human's inbox with no sign it
had been answered.

The human, verbatim (2026-09-29): "This is a weird situation where you respond to the
question on my behalf, but the message sits open for me; would like to make that smoother
somehow".

## Proposal (KISS)

- **A reply closes the question.** When anyone the human has delegated to (the orchestrator;
  maybe the advisor) sends a message with `reply_to: <question id>`, the daemon marks that
  question answered: `answered_by` and the reply's id on the question.
- **The human's inbox shows it as answered**, not open: "answered by orchestrator: <first
  line>", out of the unread count, still readable in full (so the human can overrule).
- **The CLI**: `bridle send <asker> --reply-to <id> "..."`, or `bridle answer-message <id>
  "..."`, which sends to the asker and closes the question in one step.
- Who may answer for the human: a config list (default `["external:orchestrator"]`), not any
  agent, so a worker can't close the human's questions.

## Also

- The roles already say questions a rule answers go to the orchestrator, not the human
  (told to pm-1 in m-1211). The product manager's role prompt could say so explicitly.
