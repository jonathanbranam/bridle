---
id: br-3n4j
title: The orchestrator holds and asks about a ticket without putting the reason or the question on its thread
kind: fix
created: 2026-10-19
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-7ytg]
---

From the human, via track-web's aide (m-17255, 2026-10-19), verbatim: "Send this issue with the ticket
to advisor/product-manager on bridle as an issue with tracking - all questions about a ticket should
add comments in the ticket thread from the agent not only messages. This is given to manager and
workers but maybe the orch doesn't have the proper instructions."

The case: track-web ticket tw-t4wa (Play card on home). At 06:23 ET the orchestrator held it with only
this on the thread: "hold for now. I've asked the human, via aide, for the client-home scope exception,
and I'll tell you their answer." The reason (the ticket touches client-home, outside track-web's
client-games-only trial scope, `.bridle/rules/scope.md`) and the question were never on the thread, and
no question reached `external:aide` on track-web either. The human read the ticket: "What is the
issue? I can't find it reading the ticket at all. Before moving on I want to see that the issue is
written somewhere on the ticket trail I can see." Aide wrote the reason and options on the thread by hand.

What the advisor found (read-only): rule `talk-on-the-task` already names the orchestrator in its
`roles:`, and `bridle ticket hold` requires a `--reason`. But the orchestrator's role text
(`workflow/base/roles/orchestrator.md`, "You reach the human only through `external:aide`") says to
send aide a question by plain `bridle send external:aide "From orchestrator: ..."`, with no `--task`.
The manager's role says `bridle send <agent> --task <id> "..."` (the text lands on the thread). Also,
the orchestrator runs in `<host>-home`, so a plain send reaches the home project's aide, not the
ticket's project's aide, which matches "no question reached external:aide on track-web".

## The ask

1. Whenever any agent, the orchestrator included, holds a ticket or asks a question about it, the
   reason and the full question go on that ticket's thread, readable without any message. A hold
   reason says what the issue is ("touches client-home, outside the client-games-only scope in
   .bridle/rules/scope.md"), not only "asked the human".
2. Fix the orchestrator's role text: a question or hold about a ticket goes with
   `--task <id>` (or `bridle ticket ask`) on the ticket's own project (`--project <p>`), so it lands on
   the thread and reaches that project's aide. Plain sends stay for what is not about one ticket.
3. Check that `bridle send --project <p> --task <id>` works from `<host>-home` to another project's
   ticket; if not, make it work (or say what the orchestrator should run instead).
4. Check aide's and project-manager's role text for the same gap and fix it the same way.

## Proposal

Role-text change in orchestrator.md (and any role with the same gap), naming rule `talk-on-the-task`
at the point where it sends questions. Add one sentence to `talk-on-the-task`: a hold's reason and a
question about a ticket must state the issue itself. Code only if item 3 finds the cross-project
`--task` send does not work.
