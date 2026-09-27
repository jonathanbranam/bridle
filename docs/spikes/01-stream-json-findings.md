# Spike 01 findings — stream-json client

Claude Code version: **2.1.283** (same as the brief)   Date: 2026-09-27   Rust: 1.98.1 (edition 2024)
Total spike usage: all Haiku 4.5, about **$0.40 list-price equivalent** including re-runs ($0.22 is in the
committed fixtures). The five-hour window went from 5% to 7% during the session, but that figure also
includes the orchestrating session's own (Opus) usage.

Code: `spikes/stream-json/` (`cargo run -- <s1…s11|s8probe|catalogue|all>`). Fixtures: `spikes/stream-json/fixtures/`,
scrubbed with `spikes/stream-json/scrub.py`. Fixture line numbers below are 1-based lines in those files.

## Verdict

**Yes, with caveats.** A ~550-line Rust client (`agent.rs`, `events.rs`, `log.rs`; the scenarios are another ~670 lines) drove headless Claude Code through every scenario with
no deadlocks. It handled multi-turn sessions, waking from idle, interrupting mid-tool, resuming in a new
process, clean shutdown, and hook injection. The tolerant parser handled all 164 captured events without
falling back. The caveats are about semantics, not feasibility:

- A message sent mid-turn is **folded into the running turn** at the next tool boundary. It is not queued
  as its own turn.
- The interrupt receipt's `still_queued` / `cancelled` lists stayed empty even when a message was
  pending. `cancel_queued: true` silently drops that message.
- **Bash tool processes run in their own process group.** `killpg` on claude's group does not reach them
  directly: claude cleans them up on SIGTERM, but a SIGKILL orphans them.
- By default the per-cwd system-prompt sections break the prompt cache across worktrees.
  `--exclude-dynamic-system-prompt-sections` recovers most of it.

None of these rules out the design. They change what bridle's AgentHost must do (see
Recommended API).

## Answers

| # | Question | Answer | Evidence |
|---|---|---|---|
| 1 | Async Rust drive, multi-turn, no deadlocks? | **Yes.** Separate tasks for stdout, stderr and stdin; unbounded channels; control responses correlated by `request_id`. Up to 5 turns and 3 control requests in one process (S4). | all fixtures; `s4.jsonl` 3–61 |
| 2 | Idle message starts a new turn? | **Yes**, after 15 s idle, on the same `session_id`. `system/init` is **re-emitted before every turn**, not only at startup. No events arrive while idle. | `s2.jsonl` 8 (result 1), 12 (send), 13 (init again), 16 (result 2) |
| 3 | Message sent mid-turn? | **Folded into the current turn**, not queued as its own. With `--replay-user-messages` its echo appears only when it's consumed: after the running tool's `tool_result`, before the next model call. There's **no ack at send time**. There is one `result` ("DONE\nBANANA", `num_turns: 2`, `queued_turn_count: 0`). Not tested: a mid-turn message during a turn that makes no tool call. | `s3.jsonl` 6 (sent at 2.0 s), 13 (tool_result 14.4 s), 14 (echo), 17 (single result) |
| 4 | Interrupt mid-tool? | **Yes.** The receipt arrives within ~5 ms: `{"still_queued":[]}`. The turn ends with `result/error_during_execution`, `is_error: true`, `terminal_reason: "aborted_tools"`, `stop_reason: "tool_use"`, preceded by a synthetic rejected `tool_result` and a `[Request interrupted by user for tool use]` user message. The `sleep` process is gone within ~1.3 s. The session stays usable (next turn: "AFTER"). **Queued-message semantics:** a message sent during the tool call, followed by a plain interrupt, **runs as the next turn** (part c). With `cancel_queued: true` it is **dropped silently**: no echo, never run, and the receipt still says `cancelled: []`. | `s4.jsonl` 13, 15–17, 25 (a); 34–40 (b); 50–61 (c) |
| 5 | Close stdin? | Exits in **~470 ms** with **no further events**. The exit code is **0 if the last turn succeeded and 1 if the last turn's result was `is_error`** (e.g. interrupted), with nothing on stderr. | `s5.jsonl` 8–13 (0); 27–33 (1) |
| 6 | SIGTERM to the group? | Exit **143** in ~360–420 ms. The only event after the signal is a `tool_result` "Exit code 137" (claude SIGKILLs its Bash child); **no `result` event**. Nothing is left afterwards. **But `sleep` had its own pgid** (pgid 85081 ≠ claude's 85323), so claude cleaned it up, not `killpg`. With **SIGKILL** the `sleep` survives as an **orphan**. | `s6.jsonl` 11–16 (TERM); 26–31 (KILL, orphan pid in 30) |
| 7 | `--resume` in a new process? | **Yes.** It answered "PELICAN" with the **same session id** in both `init` and `result`. Passing `--session-id` together with `--resume` fails at startup (`Error: --session-id can only be used with --continue or --resume if --fork-session is also specified.`, exit 1, no stdout events). | `s7.jsonl` 26; 36 (stderr) |
| 8 | Usage data? | `result.usage` is **per turn**, summed over the turn's API calls. `total_cost_usd` and `modelUsage` are **cumulative per session and carried across `--resume`**. `rate_limit_event` appeared **once per process**, on the first API response, never again in later turns. Limits **can be queried on demand** with the undocumented control request `get_usage` (and `get_context_usage` / `get_session_cost`), with no model call. | `s2.jsonl` 8, 16; `s7.jsonl` 12, 26; `s8probe.jsonl` 4, 7, 10 |
| 9 | Capabilities? | `["interrupt_receipt_v1","interrupt_cancel_queued_v1","msg_lifecycle_v1","mcp_read_resource_v1","mcp_tool_ui_meta_v1"]`. There are 30 tools (names only in `init`), `init` is 3.6 KB, and the context breakdown is system prompt 6.5k + tools 9.6k (+15.9k deferred) + skills 2.0k tokens. | `s1.jsonl` 4; `s8probe.jsonl` 7 |
| 10 | `--append-system-prompt-file` + caching? | It **takes effect** (the model answered the codename, which only the file contains). **In one process**, turn 2 reads the whole prefix (22,867 read, 263 written). **Across processes in the same cwd**, identical requests hit fully. **Across different cwds (bridle's worktree case)**, the prefix breaks after 13,694 tokens, so the role file is re-written for every worker (9,182 written). With `--exclude-dynamic-system-prompt-sections`, 18,997 are shared and 3,740 written. | `s10.jsonl` 12, 18 (A); 31 (B); 70/83 (default, wt-1/wt-2); 96/109 (excl-dynamic) |
| 11 | Hook `additionalContext` mid-turn? | **Yes.** The agent replied "The secret word is **HERON**." The hook input has exactly `session_id, transcript_path, cwd, prompt_id, permission_mode, hook_event_name, tool_name, tool_input, tool_response, tool_use_id, duration_ms`, and **no `agent_id`** for the main agent. Latency: the script itself took 21 ms, but `hook_started` → `hook_response` took ~900 ms. Hook children see `CLAUDE_PID`, `CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_CHILD_SESSION=1`, `CLAUDE_PROJECT_DIR`, `CLAUDE_CODE_ENTRYPOINT=sdk-cli`, a per-process `CLAUDE_CODE_MESSAGING_SOCKET`, and bridle's own `BRIDLE_AGENT`. | `s11.jsonl` 12–17; `s11-hook-input.txt` |
| 12 | Robust Rust API shape? | See Recommended API. Type a few events (`init`, `result`, `control_response`, `rate_limit_event`, assistant/user content blocks), keep everything else as raw `Value`, and key the state machine on `result`, not on message text. | `src/events.rs`, `src/agent.rs` |

## Event catalogue

Observed, from `cargo run -- catalogue` over all fixtures. All lines are single JSON objects and none
failed typed parsing.

| type/subtype | count | trimmed example / notes |
|---|---|---|
| `system/init` | 24 | `{"type":"system","subtype":"init","session_id":"…","model":"claude-haiku-4-5-20251001","tools":[30 names],"capabilities":[…],"cwd":…,"permissionMode":"dontAsk","claude_code_version":"2.1.283",…}`. Other keys: `agents, analytics_disabled, apiKeySource, fast_mode_*, mcp_servers, memory_paths, messaging_socket_path, output_style, per_turn_effort_active, plugins, skills, slash_commands, terminal_slash_commands, uuid, view_mode`. **Once per turn.** |
| `assistant` | 52 | **One event per content block**, with the same `message.id` repeated: `{"type":"assistant","message":{"id":"msg_…","content":[{"type":"thinking","thinking":"","signature":"…"}]},…}`, then `…"content":[{"type":"text","text":"OK"}]…` or `[{"type":"tool_use","id":"toolu_…","name":"Bash","input":{"command":"sleep 20","description":"…"}}]`. Thinking text is empty (signature only). |
| `user` | 16 | tool results: `"content":[{"type":"tool_result","tool_use_id":"…","content":"hi","is_error":false}]`; replay echoes: `"content":"Also reply BANANA"` (plain string); interrupt marker: `[{"type":"text","text":"[Request interrupted by user for tool use]"}]` |
| `result/success` | 19 | `{"subtype":"success","is_error":false,"num_turns":1,"result":"OK","session_id":…,"total_cost_usd":0.0168,"usage":{…},"modelUsage":{"claude-haiku-4-5-20251001":{…,"costUSD":…}},"terminal_reason":"completed","stop_reason":"end_turn",…}`. Extra keys: `api_error_status, duration_api_ms, first_content_frame_ms, permission_denials, queued_turn_count, result_index` (increments per turn within a process), `subagent_stats, time_to_request_ms, ttft_ms, ttft_stream_ms, uuid, fast_mode_*` |
| `result/error_during_execution` | 3 | as above, with `is_error:true`, `terminal_reason:"aborted_tools"`, `stop_reason:"tool_use"`, and no `result` text |
| `rate_limit_event` | 18 | `{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","resetsAt":1790533200,"rateLimitType":"five_hour","overageStatus":"rejected","overageDisabledReason":"org_level_disabled","isUsingOverage":false,"unifiedWindows":{"five_hour":{"utilization":0.06,"resetsAt":…},"seven_day":{"utilization":0.13,"resetsAt":…}}}}` |
| `control_response` | 7 | `{"type":"control_response","response":{"subtype":"success","request_id":"bridle-2","response":{"still_queued":[],"cancelled":[]}}}` |
| `system/task_started` | 1 | `{"subtype":"task_started","task_id":"bi94kl5cd","tool_use_id":"toolu_…","description":"Sleep for 8 seconds","is_backgrounded":false,"task_type":"local_bash"}`. Seen only in S3, about 3 s into the `sleep`. The 20 s sleeps were interrupted or killed ~3.9 s after `tool_use`, before any `task_started`. My guess (untested) is that it's emitted only for longer-running Bash commands. |
| `system/task_notification` | 1 | `{"subtype":"task_notification","task_id":…,"status":"completed","output_file":"","summary":"Sleep for 8 seconds"}` |
| `system/thinking_tokens` | 12 | `{"subtype":"thinking_tokens","estimated_tokens":100,"estimated_tokens_delta":50,…}`. A progress tick during longer thinking. Not in the brief. |
| `system/hook_started`, `system/hook_response` | 1 each | only with `--include-hook-events`: `{"subtype":"hook_started","hook_id":…,"hook_name":"PostToolUse:Bash","hook_event":"PostToolUse"}`, `{"subtype":"hook_response",…,"output":"{\"hookSpecificOutput\":…}"}` |

**Not observed:** `system/api_retry`, any `rate_limit_event` other than `allowed`, `result/error_max_budget_usd`
and other `error_*` results, `control_request` from claude (no permission prompts in `dontAsk`), and
non-JSON stdout. The binary contains many more `system` subtypes (`compact_boundary`, `api_retry`,
`turn_starting`, `turn_preempted`, `session_state_changed`, `post_turn_summary`, `model_fallback`, …).
The catch-all is necessary.

## Surprises

1. **Mid-turn messages are folded in, not queued** (S3). The brief expected a separate queued turn or an
   ack. Instead the message becomes part of the running turn at the next tool boundary, and it produces no
   `result` of its own. Bridle can't count results to know a message "ran". A message sent mid-turn
   does become its own next turn if the current turn is interrupted (S4c).
2. **The interrupt receipt doesn't report messages that are pending mid-turn.** `still_queued: []` and
   `cancelled: []` in every case, including when a message was demonstrably pending (S4b, S4c).
   `cancel_queued: true` discarded it with no trace. Whatever those lists track, it isn't these messages.
   `queued_turn_count` was also 0 throughout.
3. **Tool processes live in a separate process group** (S4, S6). `process_group(0)` + `killpg` is not enough
   on its own. SIGTERM works because claude kills its children. SIGKILL leaves `sleep 20` orphaned.
4. **The first Bash call in a process takes ~3.6 s before the command starts.** Claude Code runs a
   `zsh -l` "shell snapshot" that **sources the user's `~/.zshrc`** (`s4.jsonl`, first process snapshot).
   Workers inherit the user's login shell environment, and the first tool call is slow.
5. **The exit code reflects the last turn.** After stdin closes it's 1 if the final `result` was an error,
   with nothing on stderr (S5b).
6. **Assistant output is split one content block per event** (thinking, then text or tool_use) sharing
   a `message.id`. Consumers must not treat each `assistant` event as a whole message.
7. **`system/init` repeats before every turn** (the brief hinted at this). It's a usable turn-start marker, but it
   carries the full ~3.6 KB init each time.
8. **Cumulative cost survives `--resume`.** `total_cost_usd` and `modelUsage` restart from the resumed
   session's totals, not from zero (S7 B: `modelUsage.inputTokens` 20 = 10 + 10).
9. **`rate_limit_event` shape differs from the SDK types.** There's no top-level `utilization`; it's per
   window under `unifiedWindows`. `get_usage` reports the same thing as **percent (0–100) with ISO
   timestamps**, while the stream uses a **0–1 fraction with epoch seconds**.
10. **The cwd breaks the cross-worker prompt cache** unless `--exclude-dynamic-system-prompt-sections` is set (S10).
11. **Hook overhead is ~0.9 s per invocation** even though the script ran in 21 ms (S11).
12. Harness note: the spike itself ran inside a Claude Code session, whose `CLAUDE*` env vars (`CLAUDECODE`,
    `CLAUDE_CODE_SESSION_ID`, messaging socket/token, …) would otherwise be inherited by the child.
    `AgentProcess::spawn` strips them. Bridle should do the same, because it may itself be launched from a
    Claude Code terminal.

## Usage data

- **Per turn:** `result.usage` = `{input_tokens, output_tokens, cache_creation_input_tokens,
  cache_read_input_tokens, output_tokens_details.thinking_tokens, server_tool_use, …}`, summed over the
  turn's API calls (S3: 2 calls, 35,008 cache-read). Timing: `duration_ms`, `duration_api_ms`, `ttft_ms`.
- **Cumulative per session:** `total_cost_usd`, and `modelUsage[model]` with `inputTokens, outputTokens,
  cacheRead/CreationInputTokens, thinkingTokens, costUSD, contextWindow, maxOutputTokens, costBasis:"list"`.
  Resume restores them. Bridle should store per-turn `usage` and treat cost as a running total to diff.
- **Rate limits in the stream:** one `rate_limit_event` per process, on the first API response. Nothing
  changed state during the spike, so there's no evidence yet about change-triggered emission.
- **Rate limits on demand:** the control request `{"subtype":"get_usage"}` works on an idle process with no
  model call (~1.1 s). It returns `rate_limits.{five_hour,seven_day}.{utilization (percent),resets_at}`, a
  `limits[]` list (`kind: session | weekly_all | weekly_scoped`, `percent`, `severity`, `is_active`),
  `extra_usage`, `spend`, `subscription_type`, and per-account usage `behaviors`, which I scrubbed from the
  fixture. `get_context_usage` gives a per-category token breakdown and the auto-compact threshold (167,000).
  `get_session_cost` returns preformatted text. `get_status` includes **account email and org name**, so
  it must be scrubbed. None of these are documented, and the request shapes (just `subtype`) were a guess
  that happened to work.
- **Cache (S10, Haiku; the final fixture is the third run of S10):**

  | case | cache write | cache read |
  |---|---|---|
  | turn 1, first ever run of this prefix (console output of run 1, not in fixture) | 9,173 | 13,694 |
  | same process, turn 2 | 263 | 22,867 |
  | new process, same cwd + file + first message | 0 | 22,858 |
  | new process, same cwd + file, **different first message** (run 1, console) | 3,614 | 19,244 |
  | **different cwd per worker**, default | **9,182 each** | 13,694 |
  | **different cwd per worker**, `--exclude-dynamic-system-prompt-sections` | **3,740 each** | 18,997 |

  For design §11.4 rule 2 (stable prefix): the role file *is* cached, but by default only for workers in
  the same cwd. For worktree workers the flag saves about 5.4k cache-written tokens per spawn. The
  remaining ~3.7k per spawn are the dynamic sections, moved into the first user message, plus the message.

## Recommended API for bridle's AgentHost

What held up from §4.2, and what changes:

```rust
pub struct SpawnConfig {
    pub cwd: PathBuf,
    pub session: SessionStart,              // New(Uuid) | Resume(Uuid) | Fork { from: Uuid, new: Uuid }
    pub model: String,
    pub extra_args: Vec<String>,
    pub env: Vec<(String, String)>,         // after stripping inherited CLAUDE* vars
    pub transcript: Transcript,
}

pub struct AgentProcess { /* child, stdin mpsc, events mpsc, pending: HashMap<request_id, oneshot> */ }

impl AgentProcess {
    pub async fn spawn(cfg: SpawnConfig) -> Result<Self>;
    pub fn send_user(&self, text: &str) -> Result<()>;               // non-async: enqueue to the writer task
    pub fn control(&mut self, req: Value) -> Result<oneshot::Receiver<Value>>; // generic, correlated
    pub async fn interrupt(&mut self) -> Result<InterruptReceipt>;   // never cancel_queued (see Surprises 2)
    pub async fn get_usage(&mut self) -> Result<Value>;              // raw; undocumented shape
    pub async fn next_event(&mut self) -> Option<Event>;
    pub async fn wait_for(&mut self, t: Duration, pred: impl FnMut(&Event) -> bool) -> Result<Event>;
    pub async fn close_stdin(self, t: Duration) -> Result<(ExitStatus, Vec<Event>)>;
    pub fn signal_group(&self, sig: Signal) -> Result<()>;
    pub fn tool_process_groups(&self) -> Vec<Pid>;                   // NEW: see below
}
```

- **Typed:** `system/init` (session_id, model, tools, capabilities, as a turn-start marker),
  `result` (subtype, is_error, terminal_reason, session_id, usage, total_cost_usd), `control_response`
  (request_id, subtype, response), `rate_limit_event` (keep `rate_limit_info` raw inside), and the content
  blocks `text` / `tool_use` / `tool_result`. **Everything else stays `serde_json::Value`** with the raw line
  kept, and parse failures degrade to `Unparsed`, never an error. That held for all 164 events.
- **Turn state machine:** idle → (`send_user`) → running (`init` seen) → idle on `result`. A `send_user`
  while running means "inject into this turn", not "queue a turn". If bridle needs "run this next as its
  own turn", it has to hold the message until the `result` itself.
- **Delivery ack:** use `--replay-user-messages`. The echo marks the moment the model will see the
  message. This is the only delivery signal available.
- **Shutdown:** close stdin, then wait up to ~5 s, then SIGTERM the group, wait 3 s, then SIGKILL, then
  **sweep tool process groups**. The sweep needs to find descendants by ppid walk (or by stamping an env
  var bridle can search for), because tool commands are not in claude's pgid. On Linux a cgroup per worker
  would make this robust.
- **Exit code:** 0/1 after stdin close means the last turn succeeded or failed. It is not a crash signal.
  Crashes and startup errors show as stdout EOF with no `result`, plus stderr (see the S7 C case). Capture
  stderr always.
- **Session ids:** `--session-id` for new sessions. For resume, pass only `--resume`; the id is preserved.
  Use `--fork-session` to get a new id.
- Changes from §4.2: `send_user` doesn't need to be async; `interrupt` can't return queued-message
  information that's actually useful; `wait_for_result` should be a special case of a generic `wait_for`.

## Risks and follow-ups

- **Version pinning.** Everything here is 2.1.283. The control protocol (`interrupt`, `get_usage`, …),
  `rate_limit_event` shape, block-per-event splitting and mid-turn folding are undocumented or semi-internal.
  Bridle should pin the version, check `init.claude_code_version` and `capabilities`, and rerun these
  fixtures as a contract test on upgrade (`cargo run -- all`, then diff the catalogue).
- **Mid-turn semantics** need a follow-up: a mid-turn message during a turn with no tool calls (pure
  generation), several mid-turn messages, and what `still_queued` / `cancelled` actually track.
  `msg_lifecycle_v1` may expose lifecycle events that weren't enabled here; the `initialize` control
  request (seen in the binary) may be how it's enabled.
- **Process containment** (Surprise 3) must be solved before bridle can guarantee cleanup. Check
  descendants with background Bash (`run_in_background`) too, and the "killed ~5 s after final result"
  claim, which this spike didn't test.
- **Worker shell environment:** Bash sources the user's `~/.zshrc` via the snapshot. Decide whether
  workers should get a controlled `SHELL`/rc for reproducibility and startup time.
- **Hook latency** (~0.9 s each) matters if bridle uses `PostToolUse` hooks for message delivery on every
  tool call. Measure with a matcher-scoped hook and compare with `--replay-user-messages`-acked stdin
  injection, which S3 shows already reaches the agent mid-turn.
- **Prompt cache:** use `--exclude-dynamic-system-prompt-sections` for workers. Check how
  `--system-prompt-snapshot` (default on) interacts with a role file changing between resumes.
- **Budget errors, API retries and rate-limit rejections** weren't observed. They need a forced test,
  e.g. `--max-budget-usd 0.001` for `error_max_budget_usd`.
- **Accounting:** `get_status` and `get_usage` return account-identifying data. Anything bridle logs
  from them needs scrubbing, and the fixture scrubber (`scrub.py`) is a starting point.
