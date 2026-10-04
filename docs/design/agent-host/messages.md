# Messages and delivery

A message has a sender principal, a recipient (an agent, `human`, or an
`external:<name>` principal — [[docs/design/agent-host/principals.md|principals]]; or
`role:<name>`, which fans out one message per live agent of that role, 404 if none),
a kind (`note`, `question`, `answer` or `task_update`; `answer` is written by `task answer`, `task_update` by the daemon: one line about one change to a task the recipient watches, never sent to whoever made the change; a client that doesn't know a kind reads it as `note`), an optional `reply_to` naming the message it
answers, an optional `task` (the body is then written in full as a note on that task's thread, and the recipient gets a short `<id>: comment added` message with the first line; an unknown task is a 404 and sends nothing), a body and a delivery state:

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

**A delegate's reply closes the human's question.** A message whose `reply_to` names a message
addressed to the human, sent by a principal in `[messages] answer_for_human` (default
`["external:orchestrator"]`; no agent unless listed), marks that message `read` and records
`answered_by`, `answered_reply` (the reply's id) and `answered_line` (its first line) on it. It
leaves the unread count; the human still reads it in full (`bridle inbox --all`, `inbox show`) and
can overrule. The inbox and TUI show "answered by <who>: <first line>". Nothing un-answers it.

The sender reads `human`, `agent w1` or `external orchestrator`.

`--when` chooses the timing:

| `--when` | Agent idle | Agent working |
|---|---|---|
| `now` *(default)* | written at once; starts a turn | written at once; **folded into the current turn** at the next tool boundary (spike 01 S3). If the turn is interrupted first, the message runs as the next turn (S4c) |
| `idle` | written at once | held by bridle; written when the turn's `result` arrives, so it starts a turn of its own. One held message is written per turn end, oldest first |

A message to an *idle* agent is also held, not written, while the budget
governor isn't `normal` (usage-and-budget.md, hold_at): writing it would
start a new turn, which the governor isn't letting happen yet. Since the
agent is already idle, it has no turn of its own ending to trigger delivery
once the governor recovers; the governor's recovery to `normal` delivers
each idle agent's oldest held message itself (usage-and-budget.md,
Resuming).

Hooks aren't used for delivery: a stdin message already reaches a working agent
mid-turn, and each hook costs about 0.9 s (S11).

A message sent while the agent is generating *without* tool calls is expected
to be taken at the turn's end. That isn't verified:
[[mid-turn-message-during-tool-less-generation-akjw|spike akjw]].

## System notices

Bridle itself sends `note`s from `system`:

- **`Context handoff:`** (`when now`), to an agent past its `[context] wind_down_at`
  ([[agents#Renewing|renewing]]).
- **`main moved: …`** (`when idle`), to workers whose branch or claimed task didn't just land, after each
  landing, coalesced to one per agent per minute; a claimant whose declared impact overlaps the
  landing also gets `spec changed under you: …`
  ([[docs/design/coordination#Telling workers main moved|coordination.md]],
  [[docs/design/impact-and-conflicts|impact and conflicts]]).
- **`task <id> filed: …`** (`when idle`), to the running `manager` when no `project-manager` runs
  ([[docs/design/coordination#Waking the manager|coordination.md]]).
- **Conflict notices**, to both claimants (or the managers) when `impact check` opens a conflict.
- **Budget wind-down and resume notices** ([[docs/design/usage-and-budget|usage and budget]]),
  and the disk monitor's low-space note ([[operating-model#Disk monitor|disk monitor]]).

## Acks

With `--replay-user-messages`, claude echoes each stdin user message at the
moment the model is about to see it (S3). Bridle keeps a FIFO of
written-but-unacked messages per agent, and matches each echo to the oldest
one with identical text. The match sets `delivered_at`, emits `message.delivered`, and then
marks the message `read` (`message.read`, actor `system`): a message that reaches an agent's
conversation is read, with no separate step. A message whose agent exits before the ack goes back to
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

**Messages to `external:<name>`** (`bridle send external:orchestrator ...`)
land in that external principal's own inbox (`bridle inbox` run with that
principal's token), the same way `human`'s do. `to` must name an
active external principal (minted with `bridle token create`, not since
revoked) or the send 404s. There's no live process to deliver to, so
`--when`/held/written/ack don't apply — an external principal reads its
inbox on its own schedule.

## Read

**What reaches an agent's context is read, automatically.** Headless agents: at the ack above.
Interactive sessions (the orchestrator, advisors): `bridle agent wake` returns the unread
messages in full (`messages`: id, from, body) and marks them read in the same store call, so a
second wake never repeats them and none is lost; the orchestrator's `wait-for-wake` marks the
message behind each `message` wake read as it answers; `bridle inbox` run by a non-human marks
what it lists read (not what it filtered out), and `inbox show <id>` marks that one. Only the
recipient's own messages are touched. `--mark-read` still works and is a no-op for them.
Messages read before this change and left `delivered` need no migration.

**The human's reads are explicit**, as before (the inbox is their task list): their wake,
`inbox` and `inbox show` leave messages unread until `inbox read` or `--mark-read`, and only
the human can `inbox unread` (an agent is refused with 403: an unread message would just be
delivered again).

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
