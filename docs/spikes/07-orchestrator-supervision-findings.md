# Spike 07 findings: what the daemon can see of an interactive orchestrator session

Claude Code version: **2.1.284**   tmux **3.7c**   Date: 2026-09-29   Usage: Haiku, four short turns
(a few cents).

Ticket [[the-orchestrator-stays-running-fx7x|fx7x]], design in
[[docs/design/agent-host/orchestrator-supervision|orchestrator supervision]]. Everything ran in a
scratch dir (`/tmp/orch-sup-spike.*`, a git repo whose `.claude/settings.json` registered
`SessionStart` and `Stop` hooks and a `statusLine` command, each appending its stdin to a log) and
in a scratch tmux session of its own (`orchspike`, pane `%60`). The real orchestrator pane (`%39`,
`@bridle=orchestrator`) was never typed into; `tmux list-panes` on it was the only contact.

## Verdict

Every signal the design needs exists and is cheap to read. One surprise changes the design:
**`/clear` starts a new session id in the same process**, so a session id pinned at launch goes
stale. The pid doesn't.

## Answers

| # | Question | Answer |
|---|---|---|
| 1 | `SessionStart` payload | `session_id, transcript_path, cwd, scratchpad_dir, hook_event_name, source, model, session_title`. `source` was `startup` at launch and `clear` after `/clear` (with a **new** `session_id`; `resume` and `compact` sources weren't run). Fires once at launch, before any prompt, so it fires in an interactive session with no prompt typed. |
| 2 | `Stop` payload | As spike 05: `session_id, transcript_path, cwd, scratchpad_dir, prompt_id, permission_mode, hook_event_name, stop_hook_active, last_assistant_message, background_tasks, session_crons` (`scratchpad_dir` is new since 2.1.283). |
| 3 | Does `Stop` fire per turn in an interactive session? | **Yes**, once per finished turn (turns 1, 2 and after `/clear`, each with its own `prompt_id`). `Stop` means "the model stopped"; `background_tasks` (empty here, none were running; not tested with one) is where a running background command would show. Neither hook fires when the process is killed: no end-of-session hook was registered, and a killed session leaves no trace but the missing pid. |
| 4 | Hook's parent pid | The hook command's parent (`os.getppid()`) **is the `claude` pid**, the same pid the launcher recorded with `print $$` before `exec claude ...` (`exec` keeps the pid). So a `SessionStart` hook can record `pid` + `session_id` with no launcher cooperation, and the launcher's pid file is enough on its own. |
| 5 | Statusline JSON | Rendered on each UI update (3 renders for one turn, the first two before the first API call). Carries `session_id, transcript_path, cwd, session_name, model, workspace, version, cost, context_window, rate_limits, prompt_cache, ...`. `context_window` is `{total_input_tokens, total_output_tokens, context_window_size, current_usage{input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens}, used_percentage, remaining_percentage}`; before the first API call `current_usage` and `used_percentage` are `null`. Context tokens = `input + cache_creation + cache_read` of `current_usage` (10 + 4992 + 26031 = 31033 = the third render's `total_input_tokens` of that session), which is what `bridle statusline` already writes to `~/.bridle/context/<session id>` (schema: spike 04). After `/clear` the id and the count both reset. `context_window_size` was 200000 on Haiku; the token thresholds assume the 1M orchestrator model. |
| 6 | Transcript fallback | `~/.claude/projects/<cwd with / and . as ->/<session id>.jsonl`, path also given as `transcript_path` in every hook and statusline payload (so no path guessing). Each `assistant` entry has `message.usage` with the same three input fields (10 / 4992 / 26031, identical to the statusline) plus `output_tokens`. Entries are written twice per message (two `assistant` lines with the same usage, 4 ms apart): take the last, don't sum. Its mtime is a last-activity time. After `/clear` a new file starts. |
| 7 | Can `tmux send-keys` launch `scripts/claude-orchestrator` in an empty pane? | **Yes.** `tmux send-keys -t <pane> -l '<path>'` then a separate `tmux send-keys -t <pane> Enter` ran a launcher script (a scratch stand-in that wrote its pid and session id and `exec`ed `claude --session-id ... --model haiku "<prompt>"`) and the session started and ran the opening prompt. Nothing typed in the pane was echoed twice or mixed with other text. |
| 8 | Is the pane safe to type into? | `#{pane_current_command}` is the foreground process name: `2.1.284` (the `claude` binary's version string) while Claude ran, `zsh` after it was killed. `#{pane_pid}` is the pane's shell, not claude, and stays the same across relaunches. So "no claude running here" = the recorded pid is dead **and** `pane_current_command` is a shell. Both are needed: the pid alone can't tell a shell prompt from a dead session with something else running in the pane (a `less`, a human's `vim`). |
| 9 | `tmux list-panes -a -F '#{pane_id} #{@bridle}'` | Lists every pane, empty tag for untagged ones, `spiketest` for the scratch pane tagged with `tmux set -p -t orchspike @bridle spiketest`. Killing the tmux session removed the pane and its tag with it. |
| 10 | Can the launcher scope a hook to one session? | **Yes.** `claude --settings '<json with hooks>'` registered a `SessionStart` hook with no project file (checked in `-p` mode with `--setting-sources ""`; `source: startup`, parent pid = the claude pid). So `scripts/claude-orchestrator` can add the hook to its own session only. Not re-checked interactively; SessionStart fired the same way in the interactive runs above. |

## Not tested

- `SIGTERM` to the recorded pid: the scratch session exited on a plain `kill <pid>` and the pane
  went back to `zsh` (no grace period needed there). A busy turn or a Remote Control session may
  take longer; the design allows a grace before `SIGKILL`.
- `compact`/`resume` `SessionStart` sources; `Stop` with a background command running.

## Surprises

1. **The session id is not stable.** `/clear` produces a new
   `session_id` in the same process (`compact` and `resume` weren't run; assume the same), and `bridle statusline` writes the context file under the
   *current* id. `scripts/context-check.sh` reads the id pinned by `scripts/claude-orchestrator`
   into `~/.bridle-orchestrator-session`, so after a `/clear` it reads a stale file forever. The
   daemon should track "the newest context file whose statusline payload came from the recorded
   pid's session", i.e. get the id from `SessionStart`, not from the launcher.
2. **First trust prompt.** A folder Claude hasn't seen asks "Do you trust this folder?" before
   anything else, and no hook fires until it's answered. The real orchestrator's repo is already
   trusted, so a relaunch doesn't hit it; a relaunch in a new checkout would sit at the prompt
   with a live pid and no context file. The stalled-launch check in the design covers this.
3. **`pane_current_command` is a version string**, not `claude`, so don't match it against
   `claude`; match against "is a shell" (`zsh`, `bash`, `sh`, `fish`).
4. **Duplicate assistant lines** in the transcript (see 6).
