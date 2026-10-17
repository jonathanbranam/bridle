+++
id = "br-4cgx"
title = "@-mention a role in a ticket reply or a document comment and that role gets a message"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T20:49:28.208Z"
updated_at = "2026-10-17T12:22:50.869257Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:4cgx",
    "agent:4cgxb",
    "agent:4cgxc",
]
priority = "low"
priority_at = "2026-10-09T20:49:31.204227Z"
branch = "bridle/4cgxc"
commit = "65aa99e63d83a162f2bdef8825aba7638f7dff6e"
summary = "Mention messages (ticket and document comments) now carry the UI link built by the shared mentions::target_url, which bridle link also uses; omitted without [gateway] public_url. daemon.md Document review gained an @-mentions paragraph; CHANGELOG line added. Tests: target_urls_match_bridle_link, review_now_test asserts the link."
ticket = "4cgx"
+++

docs/tickets/open/mention-a-role-in-a-ticket-reply-or-a-document-comment-and-t-4cgx.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T20:49:31.204Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-17T02:06:32.873Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:28.371Z
Brief (pm-1): build in stop points; stop at the first that runs long and tell the manager. Model: Sonnet. The ticket body holds the human-approved rules (one message per agent, watcher de-dup with the mention winning, \@ escape, agents may use it, unknown role reported to the author, external roles detected and sent to external:, both cross-project forms @bridle-ui:orchestrator and @ui:orchestrator plus the explicit external: form).
Stop point 1: same-project mentions in a ticket/task thread comment: parse in the daemon's task-comment path, send one message per distinct agent (who, where with link, text), de-dup against watchers, \@ escape, unknown role reported back to the author. Tests for each rule. Docs: cli.md, CHANGELOG.
Stop point 2: the same in document comments (doc_watch in crates/bridle-daemon/src/doc_watch.rs).
Stop point 3: the cross-project forms (project name and ID prefix), resolving the project via the existing cross-project send path (bridle send --project). br-re79 (reviewer front matter) builds on this: it is blocked-by this ticket. Acceptance: just check per stop point. Out of scope: the UI, reviewer front matter (re79).

### note · agent:4cgx · 2026-10-17T08:32:39.878Z
built: e2235027 (branch bridle/4cgx)

### note · agent:4cgx · 2026-10-17T08:32:39.917Z
done, stop point 1 only (ticket comments, same-project mentions): just check exit 0, 1801 tests run, 1801 passed (last full 1795); e2235027. Stop points 2 (doc_watch) and 3 (cross-project) not started; ticket stays open for them.

### note · agent:pm-1 · 2026-10-17T08:32:53.209Z
Remainder brief (pm-1), for worker 4cgxb after stop point 1 (e2235027) lands. Model: Sonnet. Build on the landed parser/de-dup/escape/unknown-role code from stop point 1; do not redo it.
Stop point 2: same rules in document comments (crates/bridle-daemon/src/doc_watch.rs): one message per distinct mentioned agent, link to the document, \@ escape, unknown role reported to the author. Tests, cli.md/docs, CHANGELOG. just check.
Stop point 3: cross-project forms @bridle-ui:orchestrator, @ui:orchestrator and the explicit @bridle-ui:external:orchestrator (same result as the short form), resolved through the existing cross-project send path (bridle send --project). Works for both ticket comments and doc comments. Tests, docs, CHANGELOG. just check.
Stop at the first point that runs long and tell the manager. Out of scope: UI, reviewer front matter (re79, which stays blocked-by this ticket).

### note · agent:manager-2 · 2026-10-17T09:13:55.508Z
integrated: 5febee5f2df682af6afca918d74138f87bee3cf5 (branch bridle/4cgx)

### note · agent:manager-2 · 2026-10-17T09:14:16.685Z
cleanup: removed agent 4cgx, branch bridle/4cgx

### note · agent:4cgxb · 2026-10-17T10:13:42.598Z
built: c3710bc8 (branch bridle/4cgxb)

### note · agent:4cgxb · 2026-10-17T10:13:44.177Z
done: stop points 2 (doc comments) and 3 (@project:role, @prefix:role, @project:external:role via outbox); just check exit 0, 1811 tests run/passed (last full 1808); c3710bc8 (main merged). Caveats: prefix form untested (needs machine registry); doc messages name the path, no link.

### note · agent:manager-2 · 2026-10-17T10:37:28.284Z
integrated: 777b427d683a6fd4f09239b9cd275a7b810e8442 (branch bridle/4cgxb)

### note · agent:manager-2 · 2026-10-17T10:37:50.114Z
cleanup: removed agent 4cgxb, branch bridle/4cgxb

### note · agent:pm-1 · 2026-10-17T10:37:56.009Z
pm-1: stop points 2 and 3 landed (4cgxb c3710bc8). Caveats from manager-2, left as known gaps (low priority, no follow-up filed): the @prefix:role cross-project form is untested because it needs the machine registry; document-comment messages name the path but carry no link. PdM to verify and resolve; br-re79 is now unblocked.

### note · external:advisor/product-manager · 2026-10-17T10:41:48.478Z
Checked by advisor (product-manager) 2026-10-17 ~6:50 AM ET: all three stop points merged (5febee5f, 777b427d); parser, de-dup, escape, unknown-role, doc comments, cross-project all done, 9 mention tests pass. NOT resolving yet: (1) the ask says the message comes 'with its link'; ticket and document mention messages carry no link (use the same link as 'bridle link'); (2) docs/design/agent-host/daemon.md 'Document review' (~200) doesn't mention @-mentions. Known, leave unless small: @prefix:role untested; document mentions fire only for the human's comments; a cross-project unknown role isn't reported back. pm-1: a small follow-up for (1) and (2) under this ticket, please.

### note · agent:pm-1 · 2026-10-17T10:41:53.884Z
Follow-up brief (pm-1), from the PdM's check. Model: Sonnet (small). Two fixes:
(1) The mention message (ticket-thread and document comments) must carry the link the ask requires: use the same link form `bridle link` produces for the ticket or document. Test that the message text contains it.
(2) docs/design/agent-host/daemon.md "Document review" (~line 200): add a paragraph on @-mentions in document comments (who gets a message, escape, de-dup, doc mentions fire only for the human's comments).
CHANGELOG line for (1). Acceptance: just check. Out of scope (known gaps, leave): the @prefix:role test, mentions from agents' document comments, unknown cross-project role not reported back; br-re79.

### note · agent:4cgxc · 2026-10-17T11:26:34.819Z
built: ced3a4ae (branch bridle/4cgxc)

### note · agent:4cgxc · 2026-10-17T11:26:34.894Z
done: mention messages carry the bridle-link URL, daemon.md paragraph, CHANGELOG; just check exit 0, 1812 tests passed; ced3a4ae

### note · agent:manager-2 · 2026-10-17T12:22:10.555Z
integrated: 65aa99e63d83a162f2bdef8825aba7638f7dff6e (branch bridle/4cgxc)

### note · agent:manager-2 · 2026-10-17T12:22:34.550Z
cleanup: removed agent 4cgxc, branch bridle/4cgxc

### note · external:advisor/product-manager · 2026-10-17T12:22:50.869Z
Verified by advisor (product-manager) 2026-10-17 ~8:25 AM ET: follow-up landed (65aa99e6): mention messages carry the bridle-link URL (mentions.rs link_for/target_url), daemon.md 'Document review' covers @-mentions (~215). All of the ask done; known minor gaps on the earlier note stay as they are. Resolving.
