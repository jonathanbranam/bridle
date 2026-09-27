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

## Permission prompts as questions (planned)

Not built yet ([[docs/proposal/build-order|build order]] item 3). [Spike
03](docs/spikes/03-permission-prompt-tool-findings.md) settled the mechanism,
which isn't the one build-order originally assumed: claude does **not** send
a `can_use_tool` control request over the stdin/stdout channel that already
carries `interrupt` and `get_usage`. It only asks if `--permission-prompt-tool
<name>` names an **MCP tool**, served over `--mcp-config`, and the tool is
called with a `tools/call` per prompt-worthy tool use — `arguments.tool_name`,
`arguments.input`, `arguments.tool_use_id`, correlated to the `assistant`
event's `tool_use.id`. `--permission-prompts host` (already bridle's plan; it
must not be `none`) is necessary but not sufficient without that tool. This
only fires for tool calls claude's own (undocumented) risk classifier decides
are prompt-worthy — reads and in-cwd writes never reach it — and a role stuck
on `--permission-mode dontAsk` never reaches it either, whatever
`--permission-prompts` says; a role that wants questions needs a different
permission mode too (spike 03 answer 5).

The plan this unblocks:

- **Bridle serves the MCP tool itself.** A small server, reachable the way
  other per-agent MCP servers are (`--mcp-config`, `--strict-mcp-config`),
  exposing one tool that every prompt-worthy tool call goes through. The
  simplest shape is a `bridle` subcommand that speaks MCP stdio to claude on
  one side and calls back into the daemon (over its own API, the way any
  other bridle client would) on the other, so the daemon stays the one place
  that knows about agents, roles and messages.
- **A `tools/call` becomes a `question`** from the agent to its role's
  configured recipient (manager or human), reusing the existing message model
  above verbatim: same `pending → written → delivered → read` states, same
  `bridle send <recipient> --reply-to` reply path. The open MCP call blocks
  until that reply lands — spike 03 held one for 130 s with no ill effect, and
  nothing in the design needs a shorter bound than a human actually takes to
  decide. A role's rules can also answer without asking (an allow/deny rule
  keyed on tool name or command pattern), the same way `--permission-prompts
  none` already lets the permission mode decide without asking anyone.
- **The response** is the tool's decision, JSON-encoded a second time inside
  the MCP text content block (`{"content":[{"type":"text","text":
  "{\"behavior\":\"allow\"|\"deny\", ...}"}]}`) — an MCP-shape detail the
  bridging subcommand handles, not something a human answering a question
  ever sees.
- **Still open**: whether the MCP server is one process per agent or shared
  across a daemon's agents; what happens if it crashes or the daemon restarts
  mid-prompt (the pending `tools/call` has no bridle-side durability yet);
  and whether `updatedInput` (letting an allow rewrite the command, not just
  pass it through) is worth exposing to a human answering the question.
