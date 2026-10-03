+++
id = "br-00eb"
title = "Read one message in full from the CLI"
kind = "feature"
state = "dropped"
created_at = "2026-09-28T16:58:22.810Z"
updated_at = "2026-09-28T22:21:07.163184Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

ticket: docs/questions/open/read-one-message-in-full-cu5m.md
original id: cu5m
see also: fgu6 (the TUI equivalent, separate task)

The human hits this daily and called it "getting annoying" (2026-09-28); no longer low
priority in practice even though the ticket is filed that way.

The ask, verbatim: "is tehre any cli to just read a single complete inbox message? I can't
find one... i just want to see a message by name fully printed in my terminal so I can
read the full text and respond" -- and later, after finding `bridle inbox --mark-read`
marks every listed message, not one: "can't I read or mark an individual message as read?"

Brief, two small CLI additions:
1. Show one message in full: something like `bridle inbox show <id>` (or `bridle message
   <id>` -- pick whichever fits the existing `bridle inbox`/`bridle send` command
   naming better, check crates/bridle/src/cli.rs for the current subcommand shape under
   `inbox`). Print the header (from, kind, time, reply-to) and the full body, plus the
   reply command to use (`bridle send <from> --reply-to <id> "..."`), matching
   docs/design/cli.md's conventions for other `show` commands (e.g. `bridle task show`).
2. Mark one message read: `bridle inbox read <id>...` (accepts one or more ids), calling
   the existing `POST /v1/messages/{id}/read` (docs/design/agent-host/api.md) per id --
   the API endpoint already exists and already enforces recipient-only (the orchestrator
   got a 403 reading the human's messages, which is correct and should stay). `--mark-read`
   on `bridle inbox` today marks every *listed* message; this adds marking a specific one
   without listing/marking everything else.
   Whether `bridle inbox show <id>` also marks it read is open -- the ticket doesn't
   settle it; a reasonable default is yes (reading it is what marks it read in most
   inbox UIs), with a `--no-mark-read` flag if you want to keep listing cheap. Use your
   judgement, document the choice in docs/design/cli.md.

Acceptance: just check passes; a test for `bridle inbox show <id>` printing full body and
header; a test for `bridle inbox read <id>` marking exactly that message read and no
others; docs/design/cli.md updated with both commands.

Out of scope: the TUI equivalent (fgu6, separate task); any change to the read/write
permission model on POST /v1/messages/{id}/read (already correct, don't touch it).

Model: Haiku (small, mechanical CLI addition over an existing endpoint).

## Thread

### note · agent:pm-1 · 2026-09-28T22:21:07.163Z
dropped: Merged to main (2f22c02).
