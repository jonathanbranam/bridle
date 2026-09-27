# Messages and delivery

A message has a sender principal, a recipient (an agent, or `human`), a kind
(`note` or `question`), an optional `reply_to` naming the message it answers,
a body and a delivery state:

```
pending ─► written ─► delivered ─► read
   │  ▲       │
   │  └───────┘  agent exited before the ack (held messages too)
   ├─► held      (--when idle, agent working) ─► written
   └─► dropped   (interrupt --drop-held, or rm)
```

The wider message model, with task and role recipients, is in
[[docs/design/coordination#Messages|coordination]].

## Delivery

Delivery to an agent is always a stdin user message:

```
[bridle message m-0042 from human]
<body>
Reply with: bridle send human --reply-to m-0042 "<answer>"      ← questions only
```

The sender reads `human`, `agent w1` or `external orchestrator`.

`--when` chooses the timing:

| `--when` | Agent idle | Agent working |
|---|---|---|
| `now` *(default)* | written at once; starts a turn | written at once; **folded into the current turn** at the next tool boundary (spike 01 S3). If the turn is interrupted first, the message runs as the next turn (S4c) |
| `idle` | written at once | held by bridle; written when the turn's `result` arrives, so it starts a turn of its own. One held message is written per turn end, oldest first |

Hooks aren't used for delivery: a stdin message already reaches a working agent
mid-turn, and each hook costs about 0.9 s (S11).

A message sent while the agent is generating *without* tool calls is expected
to be taken at the turn's end. That isn't verified:
[[mid-turn-message-during-tool-less-generation-akjw|spike akjw]].

## Acks

With `--replay-user-messages`, claude echoes each stdin user message at the
moment the model is about to see it (S3). Bridle keeps a FIFO of
written-but-unacked messages per agent, and matches each echo to the oldest
one with identical text. The match sets `delivered_at` and emits
`message.delivered`. A message whose agent exits before the ack goes back to
`pending` and is re-delivered on resume, and so does a held one. A message to an agent that isn't
running stays `pending` until it is resumed.

Matching by text relies on claude echoing the text verbatim, which it did in
spike 01. If a future version adds a `uuid` to stdin user messages, match on
that instead. The version contract test that would catch a change is
[[pinning-and-checking-the-claude-code-version-enz3|version pinning]].

## Messages to the human and between agents

**Messages to `human`** land in the human inbox (`bridle inbox`). They are
events, so a TUI shows them live.

**Agent-to-agent** messages work the same way as human-to-agent ones, with the
sending agent as principal.

The inbox lists every message not yet `read`, so an agent's inbox also shows
messages already delivered to it over stdin until it runs
`bridle inbox --mark-read`.

## Interrupt

`bridle interrupt <agent>` sends `{"subtype":"interrupt"}` and waits for the
correlated `control_response` receipt (S4, ~5 ms; bridle gives up after
10 s). The turn then ends with `result/error_during_execution`,
`terminal_reason: "aborted_tools"`. The running tool's processes are killed by
claude, and the agent returns to `idle`.

**Bridle never sends `cancel_queued: true`**, because it silently drops
pending messages (S4b). Held (`--when idle`) messages are bridle's own queue,
and `bridle interrupt --drop-held` discards those explicitly, with a
`message.dropped` event each.
