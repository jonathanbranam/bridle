# Spike 02 findings: `--max-budget-usd`

Claude Code version: **2.1.283**   Date: 2026-09-27   Model: Haiku, about $0.03 list price.

Part of spike `mgjh` (forced budget, retry and rate-limit errors). Only the
budget cap was forced here; API retries and rate-limit rejections are still
open in that ticket. Run by hand: `claude -p` stream-json with
`--replay-user-messages` off, in an empty directory, two user messages 25 s
apart on one process, then two `--resume` runs of the same session.

## Findings

1. **The cap is checked after each model call, not before.** With
   `--max-budget-usd 0.001`, the first turn ran in full (the assistant
   replied) and cost $0.0171. Its result was:

   ```json
   {"type":"result","subtype":"error_max_budget_usd","is_error":true,
    "total_cost_usd":0.0170969,"terminal_reason":"budget_exhausted",
    "errors":["Reached maximum budget ($0.001)"],"result":null,"num_turns":1}
   ```

   So a single turn can overshoot the cap by the cost of one model call, and
   the turn's reply is in the assistant events but not in `result`.
2. **The process stays up.** The second message got a new `system/init` and
   then the same `error_max_budget_usd` result at once, with no model call
   (`total_cost_usd` unchanged). After stdin closed, the exit code was 1 (the
   last turn was an error, as in spike 01 S5).
3. **The cap is per process, not per session.** Resuming the same session
   with `--max-budget-usd 0.01`, when the session had already spent $0.017,
   succeeded: `total_cost_usd` went to $0.0202, a $0.003 turn. The resumed
   process only counts its own spend, although `total_cost_usd` stays
   cumulative across `--resume` (spike 01 S7).

## What bridle does with it

- A role's `max_budget_usd` is passed as `--max-budget-usd`.
- On an `error_max_budget_usd` result, bridle emits `agent.budget_exhausted`
  and stops the agent (exit reason `budget_exhausted`), since every later turn
  would fail without doing anything. Its messages wait, pending.
- `bridle resume` starts a new process, so it grants a fresh allowance
  ([[docs/design/agent-host/agents#Spend cap|spend cap]]).
- The contract suite checks findings 1 and 2 on every upgrade
  (`crates/bridle-claude/tests/contract_test.rs`).
