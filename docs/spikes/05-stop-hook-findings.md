# Spike 05 findings — Stop hook stdin schema and blocking mechanism

Claude Code version: **2.1.283**   Date: 2026-09-28   Total spike usage: Haiku, **~$0.05** (four short `-p`
runs, one deliberately looped 9 times to find the built-in safety cap; well under the $1 cap).

Scratch project: `/tmp/stop-hook-spike-<random>` (outside this repo, not committed). `.claude/settings.json`
registered a `Stop` hook (`python3 $CLAUDE_PROJECT_DIR/hook.py`) that logs its stdin JSON to a file, then
allows or blocks the stop per a control file, so each mechanism could be tested in isolation with tiny
prompts (`claude -p "Reply with exactly the word OK and nothing else." --model haiku --setting-sources project`).

## Verdict

**Confirmed against real `claude`.** The Stop hook's stdin schema, both blocking mechanisms, and a
built-in loop-safety cap are all pinned down below.

## Answers

| # | Question | Answer | Evidence |
|---|---|---|---|
| 1 | Stdin JSON schema | `session_id, transcript_path, cwd, prompt_id, permission_mode, hook_event_name, stop_hook_active, last_assistant_message, background_tasks, session_crons`. No `agent_id`. `hook_event_name` is always `"Stop"`. `last_assistant_message` carries the text the agent was about to stop on — useful for a hook that inspects what was said. `background_tasks` and `session_crons` were empty arrays in every run (no backgrounded tools or crons in these probes). | allow-mode capture, run 1 |
| 2 | Blocking mechanism: exit code 2 | **Works.** stderr text becomes the block reason. Claude Code prepends `[<command>]: ` to it and delivers it as a new **user** message: `"Stop hook feedback:\n[python3 $CLAUDE_PROJECT_DIR/hook.py]: BLOCK-VIA-EXIT2: please say the word PINEAPPLE before stopping\n"`. The agent read it as an instruction and replied "PINEAPPLE" on the next turn — the reason text reaches the agent as ordinary conversational content, not a system directive. | transcript `285ba4d4…jsonl`, user-message entries; hook-log `stop_hook_active: true` on retries |
| 3 | Blocking mechanism: stdout JSON | **`{"decision":"block","reason":"..."}` at the top level works**, same effect as exit 2. **`{"hookSpecificOutput":{"hookEventName":"Stop","decision":"block","reason":"..."}}` does NOT block** — the hook ran once, `stop_hook_active` stayed `false`, and the agent stopped normally on "OK", ignoring the reason. So Stop hooks use the flat `decision`/`reason` shape, unlike `PreToolUse`'s `hookSpecificOutput.permissionDecision`. | json-block run: agent replied "MANGO"; json-hookspecific-block run: agent replied "OK", hook ran exactly once |
| 4 | Repeated blocking / loop safety | Claude Code caps consecutive Stop-hook blocks **per turn at 9** by default (`system/informational`: *"A hook blocked the turn from ending 9 consecutive times — overriding and ending turn. For Stop/SubagentStop hooks, check `stop_hook_active` in the input and return success while it's true. Set `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` to raise this limit."*). A hook that ignores `stop_hook_active` and always blocks will run 9 times, cost ~$0.024 in Haiku turns, then be **force-overridden** regardless of its own decision. | transcript `285ba4d4…jsonl`, final `system/informational` entry; `system/stop_hook_summary` entries (`hookCount: 1`, `hookErrors: […]`, `preventedContinuation: false` on every one) |
| 5 | Does `stop_hook_active` actually let a hook block exactly once? | **Yes.** A hook that returns success (exit 0, no block) whenever `stop_hook_active` is `true` blocked cleanly on the first call and then let the very next Stop event (which carries `stop_hook_active: true`) through — no loop, one blocked turn, one final stop. | json-block / json-hookspecific-block runs, both single hook-log entries |
| 6 | Latency / ordering | Consecutive hook invocations in the 9x-loop run were **~1.0–1.3 s apart** (own turn + hook dispatch), consistent with spike 01's ~0.9 s hook-latency finding. No surprises in ordering: the hook always runs after the assistant's final text and before the CLI either prints it (allowed) or starts the next turn (blocked). | hook-log `received_at` deltas, run "exit2" |

## Settings shape used

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          { "type": "command", "command": "python3 $CLAUDE_PROJECT_DIR/hook.py" }
        ]
      }
    ]
  }
}
```

`matcher` is accepted but appears to be vestigial for `Stop` (there's no tool name to match on);
an empty string worked. `$CLAUDE_PROJECT_DIR` resolved to the scratch dir.

## Surprises

1. **Two different JSON shapes for "block", only one of which works.** The flat `decision`/`reason`
   documented for other hook events is what `Stop` honors; the `hookSpecificOutput` wrapper (used
   elsewhere, e.g. `PreToolUse` permission decisions) is silently ignored here.
2. **The block reason arrives as a plain user-turn message**, prefixed `Stop hook feedback:\n[<command>]: `,
   not as a distinguished system note — a blocked agent cannot tell a hook's objection apart from something
   a human said, beyond that prefix text.
3. **The 9-block override is a hidden circuit breaker**, not documented behavior found elsewhere in this
   repo's spikes. Any bridle Stop hook that can legitimately want to block more than once per turn must
   read `stop_hook_active` and stop objecting once it's `true`, or risk being silently overridden after 9
   rounds (and paying for all 9).

## Ticket disposition

`docs/spikes/open/hook-mid-turn-injection-latency-65bf.md` asked about `PostToolUse` `additionalContext`
latency, a different hook event from the `Stop` question here. Per pm-1, its ticket is reused rather than
closed as superseded: moved to `docs/spikes/done/` with a note that its original `PostToolUse` question is
superseded (delivery uses stdin now, per `docs/spikes/01-stream-json-findings.md`) and that the Stop-hook
question this spike actually needed is answered above.
