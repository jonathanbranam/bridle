---
id: n94h
title: TUI inbox: q closes an open message; opening a message no longer marks it read
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [tui-panels-and-seeing-the-work-y496]
closed: 2026-10-09T23:11:04Z
---

## The ask

The human, verbatim (2026-09-29, via the advisor), about `bridle tui`'s inbox view:

> small request: add q as option to dismiss a message;
>
> Also - wow, hitting enter marks a message as read? erg... I didn't reply or handle the message.

## Today (at be6a72d)

- `crates/bridle-tui/src/app.rs`: in an opened message, `Esc`/`Enter` close it (line 238); `q`
  only quits the whole TUI from the list (line 245). Opening a message sets `pending_mark_read`
  (line 286), so it leaves the unread list though the human hasn't handled it.
- `bridle inbox show <id>` does the same by default (`--no-mark-read` to opt out;
  `docs/design/cli.md`).

## Shape

1. `q` closes an opened message (as `Esc`/`Enter` do); `q` on the list still quits.
2. Opening a message doesn't mark it read. It's marked read when the human handles it: replying
   (`r`, as now), or an explicit key (e.g. `d`, done) on the list or in the message. Show the key
   in the title bar.
3. `bridle inbox show` matches: no mark-read by default (`--mark-read` to opt in), so reading a
   message never hides it. Update `docs/design/cli.md` and the TUI section.

## The TUI doesn't show open questions (added 2026-09-29)

The human, verbatim:

> also - I see 3 inbox messages with bridle inbox, but none in bridle tui

`bridle inbox` lists unread messages **plus every task's open question**
(`crates/bridle/src/commands.rs`, `inbox_list`, `list_open_questions`). The TUI polls unread
messages only (`crates/bridle-tui/src/run.rs`). The human had opened the three question messages
(m-1959, m-1961, m-1963) in the TUI at 01:16–01:19Z, which marked them read, so the TUI showed
nothing while the questions (br-33a3, br-c83e, br-88e1) were still unanswered.

4. The TUI's inbox also lists open questions, as `bridle inbox` does, until they're answered.

Shape, as built (br-6441): the TUI polls `GET /v1/questions` with the unread messages and shows
the questions after them (row id = task id). A question leaves only when answered: opening it,
closing it or `d` never marks anything read, and `r` skips it (answer through the task).

## Mark a message unread (added 2026-09-29)

The human, verbatim:

> also might be helpful if we can mark messages unread, if that doesn't exist yet

It doesn't: the API has `POST /v1/messages/{id}/read` only (`docs/design/agent-host/api.md`), the
CLI `bridle inbox read <id>...`.

5. `POST /v1/messages/{id}/unread` (clears `read_at`), `bridle inbox unread <id>...`, and a key in
   the TUI (e.g. `u`) on the list or in an opened message. Useful for advisors too: one that
   opens a message meant for another advisor (ervd) can put it back.
Work status: 2 integrated task(s); 1 dropped: br-180a.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
