+++
id = "br-3sta"
title = "Forwarded messages don't carry the sending project; replies route to the receiving machine's default project"
kind = "bug"
state = "integrated"
created_at = "2026-10-30T01:41:47.676Z"
updated_at = "2026-10-30T02:18:38.400035Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:s3sta",
]
branch = "bridle/s3sta"
commit = "b4d46c05ec1d9bd2591362f9a34ab1c5c4ed38af"
summary = "The forward payload already carried the sending project (origin_daemon, recorded in forwarded_in) and a bare --reply-to already routed back to it (br-eupm/br-jpph); no payload field was added, so older peers and payloads are unaffected. The remaining gap was a plain send to the label a forwarded sender shows (external:aide@m1), which went to the visitor record / default project. send_message now routes such a send (no reply_to) to the project that label last forwarded from (server.rs plain_reply_route, store.rs forwarded_project_of); no forwarded record or an unreachable project keeps old routing. Tests in outbox_test.rs; docs in messages.md; CHANGELOG Fixed."
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

### note · agent:s3sta · 2026-10-30T02:18:04.866Z
built: c6e49a4851c5bc7145164abeb5b656215834496c (branch bridle/s3sta)

### note · agent:s3sta · 2026-10-30T02:18:07.649Z
done: plain send to a forwarded sender's label (external:aide@m1) now routes to the project it wrote from; bare --reply-to already did (br-eupm/jpph) and the payload already had origin_daemon, so no new field and old peers are unaffected. just check exit 0 with BRIDLE_* unset, 1979 tests passed (last full 1972); c6e49a4851c5bc7145164abeb5b656215834496c

### note · agent:manager-2 · 2026-10-30T02:18:17.448Z
integrated: b4d46c05ec1d9bd2591362f9a34ab1c5c4ed38af (branch bridle/s3sta)

### note · agent:manager-2 · 2026-10-30T02:18:38.400Z
cleanup: removed agent s3sta, origin branch bridle/s3sta, branch bridle/s3sta
