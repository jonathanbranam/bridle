+++
id = "br-wjhp"
title = "Document review can't start an agent for a real ticket: doc-<stem> exceeds the 40-char agent name"
kind = "bug"
state = "planned"
created_at = "2026-10-04T15:00:20.270Z"
updated_at = "2026-10-04T16:02:10.232797Z"
created_by = "external:advisor/doc-review"
watchers = ["external:advisor/doc-review"]
priority = "high"
summary = "doc_watch::agent_name now names a ticket's document agent doc-<id> (stem ends -<4-char id>), and other files doc-<slug cut>-<6 hex of sha256(path)>, always within 40 chars. Tests cover a real ticket stem, long colliding stems, short stems; two existing tests that hardcoded old names now call agent_name. Docs: daemon.md Document review, CHANGELOG. Not done: log-once-per-path (needs watcher state; not cheap)."
+++

original id: wjhp
Bug, small, high. Ticket: docs/tickets/open/document-review-can-t-start-an-agent-for-a-real-ticket-doc-s-wjhp.md (read it: cause, daemon log, fix). doc_watch::agent_name (crates/bridle-daemon/src/doc_watch.rs ~57) names a document's agent doc-<whole file stem>; agent names are limited to [a-z0-9][a-z0-9-]{0,39} (worktree.rs ~22), and real ticket stems are about 60+ characters, so 'bridle review now' and the watcher fail for nearly every ticket (it blocks the human's review of 3haz). Fix: when the stem ends in a ticket id (-<id>, ticket alphabet abcdefghjkmnpqrstuvwxyz23456789, 4 chars) name the agent doc-<id>; for any other file cut the slug so the name fits in 40 chars and add a short hash of the path so two long names don't collide. Also stop the WARN-every-pass retry noise if it is cheap (log once per path per failure). Tests: a real-length ticket stem (e.g. daemons-deliver-mail-to-each-other-across-machines-store-and-3haz) gives doc-3haz and is a valid agent name; a long non-ticket stem fits in 40 and differs for two different paths; short stems still work. Docs: the 'Document review' section of daemon.md says how the name is formed; CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none (an agent already named under the old rule, if any, is not renamed; say so if relevant). Out of scope: the UI.

## Thread

### note · external:orchestrator · 2026-10-04T15:00:42.547Z
priority: normal -> high
