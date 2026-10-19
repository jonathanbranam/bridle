---
id: talk-on-the-task
severity: must
roles: [orchestrator, advisor, project-manager, manager, worker, reviewer, designer]
---
Discuss a ticket on the ticket, not in direct messages. A message is only the
notification.

- **Anything about a ticket goes on its thread**: the brief and changes to it,
  questions and answers, review findings, "sent back because ...", decisions
  made along the way, and the landing summary. Use `bridle ticket comment <id>`
  (`--text-file` for long or quoted text), `bridle ticket ask` / `bridle ticket answer`
  for questions that block it. The ticket's thread is `tickets/<id>.md` on the state
  branch; `bridle ticket show <id>` prints the ticket and its thread. The `task`
  forms still work (the only ones for a row with no ticket) until the rename.
- **State the issue itself.** A hold's reason and a question say what the problem is and what
  is being asked, in full, on the thread; "asked the human" or "see my message" is not enough.
  The human reads the ticket, not your messages.
- **Then notify with a short message that names the ticket**: `bridle send
  <agent> "tw-1234: comment added (review findings)"`. The recipient reads the
  thread with `bridle ticket show <id>`.
- **Direct messages are for what isn't about one ticket**: coordination,
  startup and ready notes, "main is red", budget and restarts.
- Lasting decisions still go where they outlive the ticket (a design doc, spec
  or rule; `record-decisions`), with a note on the ticket pointing there.

Why: the human, 2026-09-29: "if we have DMs with each other, the information
about the conversation is lost. Instead we use JIRA and make comments on the
ticket so the information is associated with that ticket after completion."
A ticket's thread is what the advisor reads to answer "what was done, and why".
