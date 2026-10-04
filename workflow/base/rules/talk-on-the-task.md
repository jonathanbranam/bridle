---
id: talk-on-the-task
severity: must
roles: [orchestrator, advisor, project-manager, manager, worker, reviewer]
---
Discuss a task on the task, not in direct messages. A message is only the
notification.

- **Anything about a task goes on its thread**: the brief and changes to it,
  questions and answers, review findings, "sent back because ...", decisions
  made along the way, and the landing summary. Use `bridle task comment <id>`
  (`--text-file` for long or quoted text), `bridle task ask` / `bridle task answer`
  for questions that block it.
- **Then notify with a short message that names the task**: `bridle send
  <agent> "tw-1234: comment added (review findings)"`. The recipient reads the
  thread with `bridle task show <id>`.
- **Direct messages are for what isn't about one task**: coordination,
  startup and ready notes, "main is red", budget and restarts.
- Lasting decisions still go where they outlive the task (a design doc, spec
  or rule; `record-decisions`), with a note on the task pointing there.

Why: the human, 2026-09-29: "if we have DMs with each other, the information
about the conversation is lost. Instead we use JIRA and make comments on the
ticket so the information is associated with that ticket after completion."
A task's thread is what the advisor reads to answer "what was done, and why".
