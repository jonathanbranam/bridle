---
id: cu5m
title: Read one message in full from the CLI
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [fgu6]
---

## The ask

The human, verbatim (2026-09-28):

> is tehre any cli to just read a single complete inbox message? I can't find one; if not,
> add a low priority ticket to add this functionlity. i just want to see a message by name
> fully printed in my terminal so I can read the full text and respond

## Notes

- There is none today. `bridle inbox` lists messages in a table; the API has no
  `GET /v1/messages/{id}` (`docs/design/agent-host/api.md`), only the list and
  `POST /v1/messages/{id}/read`.
- Workaround: `bridle inbox --all --json | jq -r '.messages[] | select(.id=="m-0890") | .body'`.
- Something like `bridle inbox show <id>` (or `bridle message <id>`) printing the header
  (from, kind, time, reply-to) and the whole body, with the reply command to use
  (`bridle send <from> --reply-to <id> "..."`). Whether showing it marks it read is open.
- The TUI equivalent is [[tui-inbox-open-a-message-in-full-fgu6|fgu6]].
- Low priority (the human).
