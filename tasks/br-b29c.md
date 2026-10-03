+++
id = "br-b29c"
title = "A delegate's reply closes the human's question (h5qd)"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T01:43:17.836Z"
updated_at = "2026-09-29T04:17:08.592310Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/answer-delegate"
commit = "ba3a032"
summary = "A reply (reply_to) from a principal in [messages] answer_for_human (default external:orchestrator) to a message addressed to the human marks it read and records answered_by/answered_reply/answered_line (migration V14, Message wire fields, AgentManager::send); it leaves the unread count. bridle inbox (list/show) and the TUI row/popup show 'answered by <who>: <first line>'. The TUI still polls unread only, so answered items drop off its list (still visible via inbox --all). No new command. Docs and PM role prompt updated. Task-level open questions are unchanged."
+++

Goal: when the orchestrator answers a question that was addressed to the human, the question closes in the human's inbox instead of sitting open. Ticket: docs/questions/open/answer-the-humans-questions-on-their-behalf-h5qd.md (read it: the incident and the proposal). Do: (1) a config list [messages] answer_for_human = [principal ids], default [external:orchestrator], names who may answer for the human; nothing else can; (2) when a message from such a principal carries reply_to pointing at a question addressed to the human, the daemon marks that question answered, recording answered_by and the reply id, and it leaves the unread and open-question counts; (3) the human's inbox and TUI show it as answered by <who> with the reply's first line, still readable in full; (4) bridle send <asker> --reply-to <id> already exists, so no new command unless it is needed. Touches crates/bridle-api/src/types.rs (wire change: update the daemon and every client together), the store and message routes in crates/bridle-daemon, and the inbox view in crates/bridle-tui. Update docs/design/agent-host (messages and API docs) and docs/design/cli.md. Add one line to the product manager role prompt (workflow/base/roles/): a question that a standing rule answers goes to the orchestrator, not the human. Acceptance: just check passes; tests for an allowed delegate closing a question, a non-listed agent not closing it, and the counts. Model: Sonnet. Out of scope: the advisor as a delegate, new commands, un-answering. Run after br-0b22 and after the TUI highlight fix, since it touches the TUI inbox.

## Thread

### note · agent:manager-2 · 2026-09-29T04:17:08.592Z
integrated: ba3a032 (branch bridle/answer-delegate)
