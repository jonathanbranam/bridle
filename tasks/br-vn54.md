+++
id = "br-vn54"
title = "Incident: remote control dropped for the dalek aide session after a tmux detach and laptop sleep; it came back only on reattach and local input"
kind = "incident"
state = "pending"
created_at = "2026-10-08T12:47:40.207Z"
updated_at = "2026-10-08T16:48:05.928361Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

Filed by the bridle aide at the human's request, 2026-10-08 ~9:00 AM ET.

## What happened
The human, verbatim: "hello - this session doesn't seem to be connecting to rc; ok this is interesting. I decided to disconnect tmux when I was moving the laptop and not sitting here - and that seems to have stopped remote control from working. Once i reconnected to tmux and interacted with this pane remote-control reconnected."

The session was bridle's aide on dalek (a laptop), running in a tmux pane. Overnight the tmux client was detached and the laptop slept. Remote control (claude.ai/code following the session) stopped working. It came back only after the human reattached tmux and typed in the pane.

## Cause: unknown
The human, verbatim: "that is very surprising to me; I didn't think that claude code would do that over tmux; it means I have to keep an attached tmux?? IDK maybe it just doesn't return after a laptop sleep. It's something to keep an eye on. On the NUC I'm not always attached to tmux and it seems to work ok there."

There are two candidates, and neither is verified:
1. A detached tmux client stops remote control. The NUC suggests not: it works there unattached.
2. Remote control doesn't reconnect after the laptop wakes until the pane gets some local input.

Category: `external` (Claude Code).

## Impact
The human can't reach the aide or orchestrator sessions on dalek from their phone after the laptop sleeps. The sessions run in tmux exactly so they can be left alone.

## Follow-up
Watch for it. Next time it happens, note:
- whether tmux was detached
- whether the laptop slept
- whether a reattach alone, without typing, brings remote control back

If it's the laptop sleep, look for a way to nudge the sessions after a wake.

## Thread

### note · system · 2026-10-08T16:48:05.927Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.
