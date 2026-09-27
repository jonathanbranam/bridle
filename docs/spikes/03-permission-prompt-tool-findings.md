# Spike 03 findings — permission prompts, `--permission-prompts` and `--permission-prompt-tool`

Claude Code version: **2.1.283** (same as spike 01)   Date: 2026-09-27
Total spike usage: Haiku, **~$0.10 list-price equivalent** across all runs (a dozen short turns plus one
~143 s held turn; the hold itself costs no tokens, only wall-clock).

Code: a throwaway Python driver and a throwaway stdio MCP server, both under `/tmp/perm-probe/` (not
committed, per the ticket).

## Verdict

**The build-order plan needs correcting, but the underlying idea works.** `--permission-prompts host`
does **not** make claude send a `can_use_tool` control request over the same stdin/stdout channel that
already carries `interrupt` and `get_usage` (spike 01). Sent alone, with no `--permission-prompt-tool`,
every prompt-worthy tool call is **auto-denied** with `system/permission_denied` — the CLI decides no
host is attached and never asks. The actual mechanism — confirmed working end to end, including allow,
deny, and a slow answer — is the older `--permission-prompt-tool <mcp-tool-name>` flag: an MCP tool,
served over `--mcp-config`, that claude calls with `tools/call` for each prompt-worthy tool use. `host`
is the correct value for `--permission-prompts` (it must not be `none`), but it's advertising *who's
answerable*, not a mechanism by itself — `--permission-prompt-tool` is the actual answerer.
`--permission-prompt-tool` doesn't appear in `claude --help`/`claude -p --help` output at all, but it is
accepted and works.

A 130 s hold before answering the MCP tool call did **not** time out the turn: it resumed and completed
normally in line with spike 01's u7pw finding that slow control-channel answers are fine.

## Answers

| # | Question | Answer | Evidence |
|---|---|---|---|
| 1 | Exact flag spelling? | `--permission-prompts <target>`, choices `host`/`none`, default `host`. Its help text says `host` means "the SDK host **or** `--permission-prompt-tool`" — that phrasing is the tell that `host` alone isn't sufficient over raw stdin. `--permission-prompt-tool <name>` is a separate, undocumented-in-`--help` flag that takes an MCP tool name (`claude -p --permission-prompt-tool foo --help` runs fine; `claude --permission-prompt-tool` with no argument fails with a normal missing-argument error, proving the flag is real, just hidden). | `claude -p --help` output; `--permission-prompt-tool` argument-missing test |
| 2 | Does claude send a `can_use_tool` **control_request** on stdout under a non-`dontAsk` mode? | **No**, not with `--permission-prompts host` alone. Tried `acceptEdits`, `manual`, `dontAsk`, with and without an explicit `initialize` control_request sent first (which claude *does* answer, listing slash commands — so the control channel itself is live), and with `CLAUDE_CODE_SESSION_ATTENDED=1` set. Every prompt-worthy Bash call (a path outside the cwd, a network `curl`) came back as a **`system/permission_denied`** event straight away, with `decision_reason_type:"other"`, and the model's own text said it "can't prompt you to approve it" in "a non-interactive session". Safe commands (`echo`) never prompt at all, in any mode — a built-in risk classifier decides what's prompt-worthy before permission mode is even consulted. | 6 driver runs, `acceptEdits`/`manual`/`dontAsk` × with/without `initialize`, all showing `system/permission_denied`, never `control_request` |
| 2b | Does it work via `--permission-prompt-tool` (an MCP tool) instead? | **Yes.** With `--mcp-config` serving one tool (`mcp__permprobe__approve_tool_use`) and `--permission-prompt-tool` naming it, the same `curl` call now triggers an MCP **`tools/call`** to that tool instead of an auto-deny. | `mcp-server.log`, `CALL` lines below |
| 3 | Request shape (the thing bridle's answerer receives)? | A standard MCP `tools/call` JSON-RPC request: `{"method":"tools/call","params":{"name":"mcp__permprobe__approve_tool_use","arguments":{"tool_name":"Bash","input":{"command":"...","description":"..."},"tool_use_id":"toolu_…"},"_meta":{"claudecode/toolUseId":"toolu_…","progressToken":2}},"id":2}`. `arguments.tool_name` is the tool being gated, `arguments.input` is its full tool input (schema depends on the tool — for Bash, `command`/`description`), and `tool_use_id` correlates to the `assistant` event's `tool_use.id`. No timestamp or session id is included; bridle would correlate by the already-running agent process. | `mcp-server.log` `RECV {"method":"tools/call",…}` in all three scenarios |
| 3b | Response shape to allow / deny, and does the turn actually proceed / block? | An MCP tool **result** whose `content` is one text block containing a JSON string (not a raw JSON object — it's double-encoded, MCP-tool-result style): `{"content":[{"type":"text","text":"{\"behavior\":\"allow\",\"updatedInput\":{...}}"}]}` to allow (claude ran `curl`, got a real HTTP response, turn ended `result/success`) or `{"content":[{"type":"text","text":"{\"behavior\":\"deny\",\"message\":\"denied by human for spike a7w8\"}"}]}` to deny (the `tool_use`'s `tool_result` came back `is_error:true` with that message as its content, the model explained it couldn't run the command, and the turn still ended `result/success`, not an error — a denial doesn't fail the turn, it just fails that one tool call). `allow` should echo back `updatedInput` (the tool's own input, unmodified in this spike); an omitted or wrong `updatedInput` wasn't tested. | `out-allow.jsonl` (curl ran, 200 in the model's answer); `out-deny.jsonl` (`tool_result` `is_error:true`, `"denied by human for spike a7w8"`, then `result/success`) |
| 4 | Does a ~2 min hold before answering time out the turn? | **No.** The MCP server slept 130 s inside its `tools/call` handler before replying. The whole process (`time claude …`) took **2:22.96** wall-clock and exited 0; the transcript shows the same `tool_use` → (130 s gap) → `tool_result` (a real curl response) → assistant text → `result/success` shape as the un-held run, just later. No error, no retry, no second `tools/call`. | `err-hold.log`/`time` (2:22.96 total); `mcp-server.log` timestamps `867.580` (CALL) → `997.588` (answered), a 130.0 s gap; `out-hold.jsonl` `result/success` |
| 5 | Does `dontAsk` bypass the tool entirely? | **Yes**, confirming agents.md's existing description. With the exact same `--permission-prompt-tool` wired up, `--permission-mode dontAsk` never called the MCP tool at all (no `RECV tools/call` in the log) — the `tool_result` came back immediately with "Permission to use Bash has been denied because Claude Code is running in don't ask mode." So switching `--permission-prompts` from `none` to `host` does nothing by itself for bridle's current `dontAsk` roles; a role also needs a non-`dontAsk` `--permission-mode` before its prompts reach the answerer at all. | `out-dontask.jsonl`; `mcp-server.log` has no `tools/call` entry for that run |

## Evidence detail

Driver: a throwaway Python script spawned `claude -p --input-format stream-json --output-format
stream-json --model claude-haiku-4-5-20251001 --session-id <uuid> --permission-mode <mode>
--permission-prompts host [--mcp-config …/mcp-config.json --strict-mcp-config --permission-prompt-tool
mcp__permprobe__approve_tool_use]`, sent one user message asking for a Bash `curl` (network access is a
prompt-worthy command; `echo` and in-cwd file ops are not — see answer 2), and read stdout until
`result`. The paired MCP server is a ~110-line stdio JSON-RPC script that answers `initialize`,
`tools/list`, and `tools/call` for one tool, reading a hold duration and a decision (allow/deny) from two
scratch files so the driver could flip behavior between runs without restarting it.

Two false leads before landing on the right mechanism, worth recording so nobody repeats them:

- **The driver's own environment was itself a Claude Code child process** (this spike runs inside a
  Claude Code session), so `CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_MESSAGING_SOCKET`, and friends leaked
  into the very first probe runs. Stripping `CLAUDE*` env vars (spike 01's surprise 12, done here with
  `env = {k: v for k, v in os.environ.items() if not k.startswith("CLAUDE")}`) made no difference to this
  finding, but it's still the right thing to do and is already bridle's documented plan.
- Sending an explicit `{"type":"control_request","request":{"subtype":"initialize"}}` before the user
  message (guessing at spike 01's unexplored "the `initialize` control request… may be how [permission
  routing] is enabled") **is answered** (a real `control_response` listing slash commands came back) but
  **did not change the permission outcome**. Whatever `initialize` does, it isn't what gates `can_use_tool`
  routing. That question from spike 01's follow-ups is now closed: `initialize` isn't the missing piece,
  `--permission-prompt-tool` is.

## Surprises / risks

1. **The build-order doc's premise is wrong for this claude version.** Item 3 says permission prompts
   arrive as `can_use_tool` control requests over the same channel as `interrupt`/`get_usage`. They don't;
   they arrive as MCP `tools/call` requests to whatever `--permission-prompt-tool` names, and that tool
   must be a real MCP server bridle stands up (a small one, per-agent or shared, is enough — this spike's
   throwaway one is a working sketch of the shape). The doc needs updating; see the design note below for
   where.
2. **A risk classifier decides what's prompt-worthy before permission mode is even consulted.** `echo`,
   reads, and in-cwd operations never reach the answerer in any mode this spike tried; only things like
   network access or paths outside the working directory did. That's good for bridle (most Bash calls
   won't become questions) but means the exact boundary isn't something bridle controls or can fully
   predict — it's internal to claude and undocumented.
3. **`dontAsk` silently skips the answerer.** Any role that should get "permission prompts as questions"
   needs its `--permission-mode` changed away from `dontAsk` (to `manual` or similar), not just
   `--permission-prompts` flipped from `none` to `host`. That's a role-config change, not just a spawn-arg
   change.
4. **The MCP tool result is double-JSON-encoded**: an MCP `content: [{type: text, text: "<JSON
   string>"}]` wrapper around the actual `{behavior, updatedInput | message}` decision. Bridle's answerer
   must serialize its decision into that inner string, not return it as a structured MCP result field.
5. **Not tested here**: `updatedInput` actually changing the command that runs (only identity round-trips
   were tried); denial reasons feeding back into the transcript/event log the way `result.permission_denials`
   currently does under `--permission-prompts none`; multiple concurrent prompt-worthy calls in one turn;
   what happens if the MCP server crashes or never answers at all (unlike this spike's bounded 130 s hold);
   and whether `--permission-prompt-tool` can be `--strict-mcp-config`'d alongside other MCP servers a role
   already has, or needs its own reserved server name.

## Bearing on the design

`--permission-prompts host` was already right to keep (it must not be `none` for this to work at all),
but the mechanism is an MCP tool bridle must serve, not a `can_use_tool` control_request bridle answers
on the existing `AgentProcess` control channel. See
[[docs/design/agent-host/messages#Permission prompts as questions|messages.md]] for the plan section this
spike unblocks.
