+++
id = "br-e9yu"
title = "Per-project sessions (aide) share one handover file and one identity across projects; key them by project"
kind = "bug"
state = "integrated"
created_at = "2026-10-05T00:21:30.006Z"
updated_at = "2026-10-05T02:42:46.805953Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
branch = "bridle/handover-by-id"
commit = "b1496f8dc52e1fd1c4971e18270b221181560b27"
summary = "Handover is now one managed record for every agent. POST /v1/handovers accepts any principal; role is the writer's identity from the token (aide, advisor/<name>, agent:<name>, orchestrator; human writes as orchestrator), project is the daemon's, never from the body. GET /v1/handovers and /latest take ?role=; CLI adds `handover list --role` and `handover latest [--role]`; prime reads role orchestrator. `bridle session aide|advisor` put the newest note's id in the opening prompt (`bridle handover show <id>`); `session restart` asks for `bridle handover write --file -` and waits for a new note of that identity; the daemon's context-limit messages name the command, not a path. Prune now keeps each role's newest. Old ~/.bridle/handover files are obsolete (noted in docs). Collision check (registry, restart, refuse_if_running) was already safe: per-project daemon and state dir; details on the thread. Docs: orchestrator-supervision, cli, storage, aide/advisor roles, CHANGELOG. Tests: handover_test (aide, named advisor, worker, filters), context-limit message assertion."
ticket = "e9yu"
+++

Ticket: docs/tickets/open/per-project-sessions-aide-share-one-handover-file-and-one-id-e9yu.md (read it; it quotes the human, including the clarification).

Approval: the human, via the bridle-ui aide (m-0259, m-0268, m-0270, 2026-10-04 ~8:25 PM ET), critical: "File is critical, but fix that". And: "That file should be based on the ID of the agent, so the named agents get a different file. I don't even know why it's a file. The orchestrator's handover is some kind of note in the system." And: "There should be a write command for an agent to write a handover, and it should be fully managed. ... This should all be managed by the system, and the system then can ensure that every agent in every project with the proper name has the right handover and that there's no confusion about anything."

Goal: aides and advisors (named advisors included) hand over through the same handover record the orchestrator uses (the `handovers` table; types.rs `Handover` has role and project), not a file under ~/.bridle/handover/. Each project's daemon keeps its own, so two projects' aides never collide, and each named advisor is its own role.
- One managed command for every agent: today `bridle handover write` / POST /v1/handovers accepts only human and external:orchestrator. Accept every principal (external sessions: aide, advisor, advisor/<name>; daemon agents: managers and workers by their agent token). The note is keyed by the daemon's project plus the writer's full identity including its name (e.g. external:advisor/doc-review, agent:manager-2), always taken from the token, never from the request body. `bridle handover show`/list can filter by that identity, and there's a way to get "the newest note for identity X" (CLI and API) for the next session. Keep the wire type unless a field is truly needed (if so, types.rs and all clients together).
- Reading: `bridle session aide` / `bridle session advisor [name]` (crates/bridle/src/session.rs ~203, ~226, take_handover) fetch the newest note for that role from the project's daemon and put it in the opening prompt (the body, or "run `bridle handover show <id>`"), instead of reading a file.
- Prompting: the daemon's context-limit message to sessions (crates/bridle-daemon/src/sessions.rs, handover_note() at ~125 and its callers) tells the session to run `bridle handover write --file -` instead of writing to a path. Drop the file path.
- Old files: leave existing ~/.bridle/handover/*.md alone (don't read or move them); one line in the doc notes they are obsolete.
- CHECK, and fix if keyed by identity alone across projects, the same collision in the session registry (sessions.rs register/lookup), `bridle session restart aide`, and refuse_if_running("aide", &project). Write what you found per item on the task thread, fixed or already safe, with the line.
- Docs: docs/design/agent-host/orchestrator-supervision.md (section 7 and the file path at ~228/~244), docs/design/cli.md (handover write/show/list), workflow/base/roles/aide.md and advisor.md where they describe handing over, storage.md if the table changes.
Acceptance: just check green; tests: aide, a named advisor and a worker each write a handover keyed by their own identity and project; a request can't set another identity; session start for aide picks up the newest aide note and not an advisor's; the context-limit message names the command, not a file.
Model: sonnet.
- No role is ever told to write a file: grep the roles, rules and daemon messages for handover/handoff file paths and point them at the command.
Out of scope (follow-up task, blocked by this one): daemon agents' renewal (supervisor.rs ~2822, the context governor's "leave a handoff note" and the "your own last handoff note" continuation) using this record. Also seats (gtzx); the waiter work (h3ar/75h2); the orchestrator's own handover flow beyond what sharing the code needs.

## Thread

### note · external:orchestrator · 2026-10-05T00:22:19.616Z
From orchestrator: br-e9yu is CRITICAL (the human, via aide: 'File is critical, but fix that'), planned with a full brief (aides and advisors hand over through the handovers record, per the human's clarification). Put it in its own tier at the very front of the queue now.

### note · external:orchestrator · 2026-10-05T00:22:19.644Z
From orchestrator: br-e9yu is critical (the human's word): every project's aide shares one handover file. Brief on the task, sonnet. Give it the next free worker slot, ahead of everything except landing what's already done (push p88z, land bnhn). Two workers max.

### note · external:aide · 2026-10-05T00:22:27.199Z
watching the task

### note · external:aide · 2026-10-05T00:22:49.318Z
The human, verbatim (via bridle-ui's aide): "There should be a write command for an agent to write a handover, and it should be fully managed. Nobody should be ready to file. This should all be managed by the system, and the system then can ensure that every agent in every project with the proper name has the right handover and that there's no confusion about anything."

### note · external:orchestrator · 2026-10-05T00:23:40.909Z
From orchestrator: brief widened per the human's m-0270 (one managed command for every agent, keyed by project + full identity incl. name; no role ever told to write a file). Daemon-agent renewal is the follow-up br-cyvf, blocked by this.

### note · agent:handover-by-id · 2026-10-05T01:09:42.546Z
Collision check (brief item "CHECK"), per item:
- Session registry (sessions.rs register/list/is_running): already safe. `Sessions::new` gets `ws.sessions_json()` (lib.rs ~556), which is `<workspace state_dir>/sessions.json`, one per project daemon; the test `registries_with_their_own_files_do_not_see_each_other` covers it. The `context/<claude session id>` files under the shared home are keyed by the unique Claude session id, also safe.
- `bridle session restart aide|advisor`: already safe. It looks the session up through `client_for(cli)`, the project's own daemon, so it only sees that project's registry.
- `refuse_if_running("aide", &project)`: already safe. It asks the project's daemon and also filters `s.project` against the project.
- The one real collision was the handover file `$BRIDLE_HOME/handover/<identity>.md` (session.rs handover_path/take_handover, sessions.rs handover_note): fixed, removed. Notes are now rows in `handovers`, keyed by role (the writer's identity from the token, @machine stripped) and the daemon's project.
Also fixed: `prune_handovers` kept only the single newest note overall, which with several roles would delete another role's latest; it now keeps each role's newest.

### note · agent:handover-by-id · 2026-10-05T01:16:40.722Z
done: handover record for every identity, no files; just check green (1184 tests); 18668824

### note · external:orchestrator · 2026-10-05T01:17:19.099Z
Orchestrator review notes for landing: (1) workflow/base/roles/orchestrator.md and the orchestrator prime still name `bridle orchestrator handover write/done/list/show`; confirm those still work (alias) or update orchestrator.md in this branch. (2) aide.md now says 'Your next session's prompt points at it': confirm `bridle session aide`/advisor prime actually prints or names the newest note for that identity (a test, or point me at it).

### note · agent:handover-by-id · 2026-10-05T01:24:12.831Z
Answers to the orchestrator's review notes:
(1) `bridle orchestrator handover write|done|list|show|latest` still work: `OrchestratorAction::Handover(a) => Command::Handover(a)` (crates/bridle/src/commands/mod.rs ~118) routes the alias to the same HandoverArgs and handler, so the same subcommands exist under both names. orchestrator.md (line ~144, `bridle orchestrator handover write --file -`) and the prime need no change; prime now reads `latest_handover(Some("orchestrator"))` (commands/orchestrator.rs ~109), so other identities' notes never appear in it. `handover done` is unchanged.
(2) Tested: crates/bridle/tests/session_test.rs `a_session_prompt_names_the_newest_note_of_its_own_identity`. A fake daemon has one note for role=aide only; `session aide` passes a prompt containing "bridle handover show h-0042", and `session advisor alice` gets no note in its prompt. The code is handover_prompt() in crates/bridle/src/session.rs, which asks `latest_handover(Some(identity))`.

### note · agent:handover-by-id · 2026-10-05T01:24:12.871Z
done: both review notes answered on the thread (alias works, new session_test); main merged, just check green (1186 tests); c4e6f703

### note · agent:manager-2 · 2026-10-05T01:32:16.076Z
main moved (br-bek3 landed, gateway code). Merge main into your branch, run just check once, then report done with exit status, test count and sha.

### note · agent:handover-by-id · 2026-10-05T01:39:42.309Z
done: main (br-bek3) merged; just check exit 0, 1191 tests run, 1191 passed, 5 skipped; aa954642

### note · agent:manager-2 · 2026-10-05T02:39:25.084Z
integrated: b1496f8dc52e1fd1c4971e18270b221181560b27 (branch bridle/handover-by-id)

### note · agent:manager-2 · 2026-10-05T02:42:46.805Z
cleanup: removed agent handover-by-id, branch bridle/handover-by-id
