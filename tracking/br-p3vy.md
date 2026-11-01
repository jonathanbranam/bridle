+++
id = "br-p3vy"
title = "Restart prompt reaches the replacement session when the old one hands over the same second"
kind = "bug"
state = "integrated"
created_at = "2026-11-01T13:04:00.514Z"
updated_at = "2026-11-01T14:49:31.019044Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wp3vy",
]
branch = "bridle/wp3vy"
commit = "5966c7ce9a282d23349b0ee302d7f52a3d34155e"
summary = "Real window found: the restart request queued AFTER the session recorded its note (the daemon's close_handover_requests only ran when the note landed). send_message in crates/bridle-daemon/src/server.rs now closes requests (marker handover::REQUEST_MARKER) to external:X when X has a note under 120 s old, same window as the CLI's 'recent' rule. Test handover_test.rs a_restart_request_queued_after_the_note_is_born_read (verified failing without the fix); the note-after-request order was already covered (br-5hca, close on write). CLI also retries mark-read once and warns. Not done: session-id scoping (schema), and timeout path leaves the request pending on purpose (session may still be alive and answer). cli.md and CHANGELOG updated."
ticket = "p3vy"
+++

docs/tickets/open/restart-prompt-reaches-the-replacement-session-when-the-old-p3vy.md

submitted by external:orchestrator

Reported by orchestrator@vesta (m-0562 on dalek-home), from aide on the notes project (m-0141 on vesta-home).

## What happened

On a restart, aide's handover prompt (notes m-0186, "The human is restarting this session", 2026-11-01 13:02:18 UTC) arrived in the same second the old aide session (pid 1321603) wrote handover h-0031, so the old session never read it. The new session's first waiter (pid 1522933) then got the prompt and was asked to hand over right after starting. Aide ignored it as stale, but a less careful session would hand over again for no reason.

## The ask

Once the handover for a session is recorded, drop the pending restart/handover prompt for that session (or mark it read) so it isn't delivered to the replacement session.

Verify: restart a session so its handover is written in the same second the prompt is queued; the new session's first wake does not include the prompt.

Priority: normal (orchestrator@vesta's recommendation).

## Thread

### note · external:orchestrator · 2026-11-01T13:04:00.515Z
submitted by external:orchestrator

### note · agent:wp3vy · 2026-11-01T14:32:08.912Z
built: 1cff5975 (branch bridle/wp3vy)

### note · agent:wp3vy · 2026-11-01T14:32:08.950Z
done: mark-read retry+warning (cause not reproduced, see summary); check exit 0, 1983 tests; 1cff5975

### note · agent:manager-2 · 2026-11-01T14:32:21.703Z
Not accepted yet: the ticket's acceptance asks for a test of the reported ordering (handover recorded first or in the same instant as the prompt is queued, then the replacement's first wake must not include the prompt), and your change is only a mark-read retry with no such test. Look for the real window: the old session writes its own handover (a --no-restart or self-written note, which ends it) while obtain_handover sends the prompt; e.g. (a) between the 'recent' check and client.send, a note lands so the loop sees it at once but the prompt is already queued: confirm mark-read really runs on every exit path of obtain_handover (early return, error, timeout); (b) after send, re-check latest_handover and, if a note newer than 'before' exists, mark read (you do); (c) also drop the request when the session ends without the note (error/timeout path). If the daemon can do it cheaply, make a daemon-side rule instead: when a handover note for identity X lands, mark pending restart-request notes to external:X read, and when such a note is queued while a handover newer than N seconds exists, queue it read. Write the failing test first, then fix. Keep the CHANGELOG line accurate. Merge main, just check with BRIDLE_* unset, commit, message me the sha.

### note · agent:wp3vy · 2026-11-01T14:49:21.674Z
done: daemon marks a restart request read when queued within 2 min after the identity's note (failing test first); check exit 0, 1984 tests; f694df77

### note · agent:manager-2 · 2026-11-01T14:49:31.019Z
integrated: 5966c7ce9a282d23349b0ee302d7f52a3d34155e (branch bridle/wp3vy)
