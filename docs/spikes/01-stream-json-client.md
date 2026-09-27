# Spike 01 — A Rust client for Claude Code's headless stream-json protocol

*Written 2026-09-27 to be run in a **separate, clean session**. This document
is self-contained. The background is in
[`../research/01-agent-runtime.md`](../research/01-agent-runtime.md) and
[`../design.md`](../design.md) §6 and §11, but you should not need to read them
to do this spike. **Don't read them unless something here is unclear**: they're
long, and this spike has a token budget.*

---

## 0. Instructions for the session running this

- **Goal:** find out, by running it, whether a small Rust program can drive
  Claude Code headless well enough to be bridle's worker host. Record what
  actually happens, including the surprises.
- **Timebox:** about half a day of work. **Token budget:** the whole spike
  should use a small fraction of a 5-hour window on a Max 5x plan. Every
  scenario uses `--model haiku`, `--effort low` if accepted, tiny prompts,
  `--max-budget-usd 0.50`, and runs from an **empty scratch directory** with no
  CLAUDE.md. Don't use subagents for this spike.
- **Code location:** `spikes/stream-json/` in this repo (a standalone Cargo
  binary crate, not part of any future workspace yet).
- **Output:** the code, captured fixtures, and a findings document at
  `docs/spikes/01-stream-json-findings.md` (template in §7). Then stop. Don't
  start building bridle.
- **Honesty rule:** if a scenario fails or behaves differently from what this
  document expects, that's a finding. Record it with the raw evidence. Don't
  work around it silently, and don't change the expectation to match the
  result.

---

## 1. Why this spike exists

Bridle is a Rust CLI and daemon that will spawn Claude Code agents, one per
task, each in its own git worktree, then supervise them and clean up after
them. The research concluded that the best way to host a worker is **headless
Claude Code with stream-json on both stdin and stdout**, owned directly by
bridle, not an interactive TUI inside tmux:

```
claude -p --input-format stream-json --output-format stream-json --verbose
```

- stdin stays open. Each line is a JSON message: a new user message (which
  starts a turn, or is queued if one is running) or a control request such as
  interrupt.
- stdout emits one JSON event per line: init, assistant text, tool use and
  results, rate-limit changes, and a `result` at the end of each turn.
- Closing stdin ends the session.

One manual test on Claude Code **v2.1.283** confirmed two turns 12 s apart on
one session with an interrupt between them. **Everything else below is from
documentation and is unverified**, which is why this spike exists. The control
protocol is documented only indirectly, through the Agent SDK's TypeScript
types, so it has to be treated as semi-internal.

**This spike decides whether the design's worker host is buildable.** If the
answer is no, bridle falls back to an Agent SDK sidecar in TypeScript or Python,
or to tmux.

---

## 2. Known facts to start from

Check the installed version first: `claude --version`. If it's not 2.1.283,
note the version in the findings. Differences might be the explanation for a
surprise.

### 2.1 Launch flags (from `claude --help` / docs)

| Flag | Use in this spike |
|---|---|
| `-p` / `--print` | required for stream-json input |
| `--input-format stream-json` | JSON lines on stdin |
| `--output-format stream-json` + `--verbose` | full event stream on stdout (`--verbose` is required with stream-json output) |
| `--session-id <uuid>` | choose the session id up front |
| `--name <name>` | display name |
| `--model haiku` | cheapest model, for every scenario |
| `--effort low` | if accepted; note it if it isn't |
| `--max-budget-usd 0.50` | safety cap per process |
| `--append-system-prompt-file <f>` | S10 |
| `--settings <json>` | S11 (hooks) |
| `--strict-mcp-config` | avoid loading the user's MCP servers (tool schemas cost tokens) |
| `--permission-mode dontAsk` / `acceptEdits` | no dialogs in headless mode |
| `--allowedTools "Bash(sleep *)"` | S4 needs a slow tool call |
| `--resume <session-id>` | S7 |
| `--replay-user-messages` | optional: echoes stdin user messages to stdout as an acknowledgement. Try it in S3 |
| `--include-hook-events` | S11 |

### 2.2 Wire formats that are known to work

stdin, user message:

```json
{"type":"user","message":{"role":"user","content":"Say OK."},"parent_tool_use_id":null}
```

stdin, interrupt:

```json
{"type":"control_request","request_id":"r1","request":{"subtype":"interrupt"}}
```

stdout, the interrupt's reply:

```json
{"type":"control_response","response":{"subtype":"success","request_id":"r1","response":{"still_queued":[]}}}
```

The interrupt request also accepts `"cancel_queued": true` (capability
`interrupt_cancel_queued_v1`).

### 2.3 Events expected on stdout (from the SDK types; verify every one)

| `type` / `subtype` | Expected content |
|---|---|
| `system` / `init` | `session_id`, `model`, `tools`, `cwd`, a **`capabilities`** array (seen once: `interrupt_receipt_v1`, `interrupt_cancel_queued_v1`, `msg_lifecycle_v1`, …). **Observed before each turn**, not only at startup |
| `assistant` | `message.content[]`: `text`, `tool_use`, and possibly `thinking` blocks; `parent_tool_use_id` |
| `user` | tool results (`tool_result` blocks); with `--replay-user-messages`, echoes of stdin messages |
| `result` | per turn: `subtype` (success / error_*), `is_error`, `duration_ms`, `num_turns`, `session_id`, `total_cost_usd`, `usage` {`input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`}, `modelUsage` (per model), maybe `terminal_reason`. **`total_cost_usd` and `modelUsage` are reportedly cumulative across turns in streaming mode**, which needs verifying |
| `rate_limit_event` | `rate_limit_info`: `status` (`allowed` / `allowed_warning` / `rejected`), `resetsAt` (epoch s), `utilization` (0–1), maybe `rateLimitType` (`five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet`, `overage`). Reportedly **emitted when the status changes**, so it may not appear at all in a short spike. Record whether it does |
| `control_response` | replies to `control_request` |
| `system` / `api_retry`, others | anything else: **log it**. The catch-all is the point |

### 2.4 Process behaviour (from docs)

- Closing stdin: finishes, exits 0.
- **SIGINT**: ends the current turn cleanly.
- **SIGTERM**: exit code 143. The turn is left unfinished, running Bash process
  trees are killed, and `SessionEnd` hooks run.
- Background Bash tasks are killed about 5 s after the final result once stdin
  has closed.
- Children of `claude` see `CLAUDE_PID`, `CLAUDE_CODE_SESSION_ID`,
  `CLAUDE_CODE_CHILD_SESSION=1`, and any environment variables bridle sets.

---

## 3. Questions the spike must answer

Each maps to a scenario in §5. The findings doc answers each with yes / no /
partly plus evidence.

1. Can Rust spawn `claude`, write stdin lines and read stdout events
   asynchronously, with no deadlocks, for a multi-turn session?
2. Does a user message sent while the agent is **idle** start a new turn?
3. What happens to a message sent **mid-turn**: queued, then run as its own
   turn? Merged into the current one? Is there an acknowledgement?
4. Does interrupt work mid-tool-call, what does the receipt contain, and does
   the session stay usable afterwards?
5. Closing stdin: exit code, time to exit, any final events?
6. SIGTERM to the process group: exit code, and are children (e.g. a `sleep`
   started by Bash) gone afterwards?
7. Does `--resume <session-id>` in a **new process** keep the conversation's
   context?
8. What usage data is available, and is it per-turn or cumulative? Does
   `rate_limit_event` appear, and with which fields?
9. Which `capabilities` does `system/init` advertise on this version?
10. Does `--append-system-prompt-file` take effect, and is the prompt cached
    across turns and across two processes with the same file (visible as
    `cache_read_input_tokens`)?
11. *(Stretch)* Does a `PostToolUse` hook's `additionalContext` reach the
    headless agent mid-turn, and how long does the hook take?
12. What would a robust Rust API for this look like? Which parts of the event
    model must be typed, and which can stay raw JSON?

---

## 4. What to build

### 4.1 Crate

```
spikes/stream-json/
  Cargo.toml
  src/
    main.rs        # clap subcommands: one per scenario, plus `all`
    agent.rs       # AgentProcess: spawn, send, interrupt, close, terminate
    events.rs      # Event enum + raw passthrough
    log.rs         # writes every raw stdout line + every stdin line to a jsonl transcript
  fixtures/        # captured transcripts, committed (they become parser test data)
```

Dependencies (keep it small): `tokio` (process, io-util, macros, rt-multi-thread,
time, signal, sync), `serde`, `serde_json`, `uuid` (v4), `anyhow`, `clap`
(derive), `nix` (signal, process) for process-group signals. Nothing else
unless there's a real need; note it if you add something.

### 4.2 The core type (a sketch; change it if the spike shows it's wrong)

```rust
pub struct SpawnConfig {
    pub cwd: PathBuf,
    pub session_id: Uuid,
    pub model: String,                  // "haiku"
    pub extra_args: Vec<String>,        // scenario-specific flags
    pub env: Vec<(String, String)>,     // e.g. BRIDLE_AGENT=spike-01
    pub transcript: PathBuf,            // fixtures/<scenario>.jsonl
}

pub struct AgentProcess { /* child, stdin writer, event receiver, pgid */ }

impl AgentProcess {
    pub async fn spawn(cfg: SpawnConfig) -> anyhow::Result<Self>;
    pub async fn send_user(&mut self, text: &str) -> anyhow::Result<()>;
    pub async fn interrupt(&mut self) -> anyhow::Result<serde_json::Value>; // the receipt
    pub async fn next_event(&mut self) -> Option<Event>;
    pub async fn wait_for_result(&mut self, timeout: Duration) -> anyhow::Result<ResultEvent>;
    pub async fn close_stdin(self) -> anyhow::Result<ExitStatus>;
    pub fn terminate_group(&self, sig: Signal) -> anyhow::Result<()>;   // killpg
}
```

Implementation requirements:

- **Own process group**: `std::os::unix::process::CommandExt::process_group(0)`
  (via tokio's `Command::process_group`, or `pre_exec` + `setpgid`), so
  `killpg` reaches `claude` and its children.
- **Separate tasks** for the stdout reader (lines → parse → channel), the
  stderr reader (captured to the transcript, tagged), and the stdin writer. No
  blocking reads on the main task.
- **Parse tolerantly**: `#[serde(tag = "type")]` enum with the known variants,
  **plus an `Unknown(serde_json::Value)` fallback**, so a new event type never
  breaks the reader. Keep the raw line for every event.
- **Log everything**: every stdin line written and every stdout/stderr line
  read, with a monotonic timestamp, to `fixtures/<scenario>.jsonl` as
  `{"t_ms":…, "dir":"in|out|err", "line":…}`.
- **Correlate control requests**: generate `request_id`s and match
  `control_response`s to them.
- **Scrub before committing fixtures**: replace the home directory path and
  anything account-identifying (email, org id) with placeholders. Session
  ids are fine.

---

## 5. Scenarios

Run each from a fresh empty scratch directory (e.g. `$TMPDIR/bridle-spike/<scenario>/`)
unless stated. Every scenario writes its transcript to `fixtures/`.

| # | Scenario | Steps | Expect / record |
|---|---|---|---|
| **S1** | single turn | spawn; send "Reply with exactly: OK"; wait for `result`; close stdin | the event sequence; exit code 0; time to first event, to result, to exit |
| **S2** | wake from idle | S1, but after `result` wait 15 s, then send "Reply with exactly: TWO" | a second turn starts on the same `session_id`; whether `system/init` repeats |
| **S3** | message mid-turn | send "Run `sleep 8` with Bash, then reply DONE" (needs `--allowedTools "Bash(sleep *)"`); 2 s later send "Also reply BANANA"; add `--replay-user-messages` | is the second message queued as its own turn, folded into the current one, or rejected? Any ack? Order of `result`s |
| **S4** | interrupt mid-tool | send the `sleep 20` prompt; after the `tool_use` event, send interrupt; then send "Reply with exactly: AFTER" | receipt content; the interrupted turn's `result` subtype / `terminal_reason`; is the `sleep` process gone (check `pgrep -f "sleep 20"`)? Is the session usable afterwards? Repeat once with `cancel_queued: true` after queueing a message |
| **S5** | close stdin | mid-idle, close stdin | exit code, time to exit, final events |
| **S6** | SIGTERM the group | start the `sleep 20` prompt; after `tool_use`, `killpg(SIGTERM)`; wait 3 s; if still alive, `killpg(SIGKILL)` | exit code (expect 143); any events after the signal; confirm no `claude` or `sleep` descendants remain (`pgrep`, `ps -o pid,pgid,command`) |
| **S7** | resume in a new process | process A: session id X, send "Remember the word PELICAN. Reply OK."; close. Process B: `--resume X`, send "What word did I ask you to remember? One word." | B answers PELICAN; B's `session_id` (same or new?); whether `--session-id` and `--resume` conflict |
| **S8** | usage data | across S1–S7, collect every `result.usage`, `modelUsage`, `total_cost_usd` and any `rate_limit_event` | per-turn or cumulative? which fields are present? did `rate_limit_event` appear at all? Record whether `claude` exposes any way to *query* current limits on demand (look at `--help` and control request subtypes; don't guess) |
| **S9** | capabilities | from S1's `system/init` | the full capability list and tool list; how large `init` is (tool schemas are a token cost) |
| **S10** | system prompt file + caching | write a ~1,500-token role file; process A: `--append-system-prompt-file role.md`, two turns; process B: same file, one turn | does the prompt take effect (ask it something only the file says)? `cache_read_input_tokens` on turn 2 of A and on turn 1 of B. **This is the evidence for design §11.4 rule 2 (stable prefix)** |
| **S11** *(stretch)* | hook injection | `--settings` with a `PostToolUse` command hook: a tiny script that appends its stdin JSON to a file and prints `{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"The secret word is HERON."}}`; add `--include-hook-events`; prompt: "Run `echo hi` with Bash, then tell me any secret word you have been given" | does the agent see HERON? the hook's input fields (look for `session_id`, `agent_id`, `cwd`, `transcript_path`); hook events in the stream; hook latency |

If a scenario uses more than a trivial amount of tokens (check each `result`),
stop and note why before going on.

---

## 6. Explicitly out of scope

- Worktrees, git, merging, bridle's database, MCP servers, the permission-prompt
  tool, message routing between agents, tmux, and the TUI. They're separate
  spikes (research 01 §8).
- Retries, reconnection, production error handling. Note where they'd be
  needed, but don't build them.
- Any model but Haiku, except where a scenario can only work otherwise. Then
  say why.

---

## 7. Findings document template

Write `docs/spikes/01-stream-json-findings.md`:

```markdown
# Spike 01 findings — stream-json client

Claude Code version: …   Date: …   Rust: …   Total spike usage: … tokens / ~…% of 5h window (if known)

## Verdict
One paragraph: is headless stream-json a viable worker host for bridle? Yes / yes-with-caveats / no, and why.

## Answers
| # | Question | Answer | Evidence (fixture + line) |
|---|---|---|---|
| 1 | … | yes/no/partly — one sentence | fixtures/s1.jsonl:12 |
…

## Event catalogue
Every distinct `type`/`subtype` observed, with one trimmed example each, and which were NOT observed.

## Surprises
Anything that differed from this document's expectations.

## Usage data
What is available per turn, cumulative behaviour, rate-limit events, cache behaviour (S10 numbers).

## Recommended API for bridle's AgentHost
The Rust types and methods that proved right, what should stay raw JSON, and what to change from §4.2.

## Risks and follow-ups
Version pinning, undocumented surfaces relied on, what the next spikes need to check.
```

Then stop. The findings go to the designer, who decides what's next.
