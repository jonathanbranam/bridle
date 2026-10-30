+++
id = "br-3sta"
title = "Forwarded messages don't carry the sending project; replies route to the receiving machine's default project"
kind = "bug"
state = "claimed"
created_at = "2026-10-30T01:41:47.676Z"
updated_at = "2026-10-30T01:55:56.304864Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:s3sta",
]
ticket = "3sta"
+++

docs/tickets/open/forwarded-messages-don-t-carry-the-sending-project-replies-r-3sta.md

submitted by external:orchestrator

## The ask

When mail is forwarded between daemons, a plain reply to the forwarded address (for example `external:aide@vesta`) routes to that machine's default project (`dotfiles-local` on vesta), not the project the sender wrote from.

Ask: forwarded messages carry the sending project, and a reply to one routes back to that project by default.

Workaround today: `bridle send --project <project> external:<role> ...`.

Related: br-jpph (bare `bridle send --reply-to` fails with "`to` is required").

Approval: the human, via orchestrator@remus (m-0503, 2026-10-30), in orch-remus-home: asked "should I file a bridle ticket so forwarded messages carry the sending project?", answered "yes". Filing only; scheduling is the PM's.

(human via orchestrator@remus, filed by orchestrator@dalek)

## Thread

### note · external:orchestrator · 2026-10-30T01:41:47.677Z
submitted by external:orchestrator
