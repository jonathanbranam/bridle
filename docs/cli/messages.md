# Messages

A message has a sender, a recipient, a kind (`note`, `question`, `answer`, `task_update`)
and a delivery state. Recipients: an agent name, `human`, `external:<name>` (an
orchestrator or advisor outside bridle), or `role:<name>` (one copy per live agent of that
role).

```
bridle send <to> "text"                      # or --text-file F; stdin with --text-file -
bridle send <to> --question "text"           # expects an answer
bridle send <to> --reply-to m-0042 "text"
bridle send <manager> --task <id> "done: ..."   # body goes on the task's thread; the
                                                # recipient gets a short pointer
bridle inbox --json                          # what's waiting for me (marks it read)
bridle inbox show <id>
```

Incoming messages show up in your conversation as a first line like
`[bridle message m-0042 from human (Jo)]`. Reply with `bridle send`.

## Timing

`--when now` (default) writes at once and folds into the agent's current turn at the next
tool boundary. `--when idle` holds the message until the turn ends. Messages to an idle
agent are also held while the budget governor isn't `normal`. A message whose agent
exits before reading it goes back to pending and is redelivered on resume.

## Talk on the task

Anything about a task goes on its thread (`bridle task comment`, `bridle send --task`);
the message is only the notification. Direct messages are for coordination that isn't
about one task.

## System notices

Bridle sends notes from `system`: `Context handoff:`, `main moved: ...` after a landing,
conflict notices, budget wind-down and resume notices.

## More

`docs/design/agent-host/messages.md`, `docs/design/agent-host/principals.md`,
`docs/design/coordination.md`.
