+++
id = "br-enx3"
title = "bridle link: the unified URL scheme (/p/{project}/tasks/{id}, ...), document and spec links; link rule with exact formats"
kind = "feature"
state = "planned"
created_at = "2026-10-08T02:22:01.310Z"
updated_at = "2026-10-08T02:22:13.463797Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Bridle side of bridle-ui's unified URL scheme (bridle-ui ui-4u8g design, ui-judt build, ui-28em part 2). The human's answers, 2026-10-07 ~10:20 PM ET in the orchestrator's session: plural kind segments; 'bridle link' also takes a document path and a spec ID; build tonight.

The exact formats are the table in bridle-ui's docs/design/url-scheme.md (written by ui-judt; read it from /Volumes/Data/work/bridle-ui-workspace/bridle-ui after it lands). Expected: {public_url}/p/{project}/tasks/{id}, /p/{project}/tickets/{id}, /p/{project}/docs?path={path}, /p/{project}/specs?path={path}#{id}.

Work:
1. crates/bridle/src/link.rs: emit the new forms. A task needs its project (project::resolve, as the ticket form already does). Accept a document path (a repo-relative path, e.g. docs/design/cli.md) and a spec ID; keep silent when public_url is unset. Tests per form.
2. workflow/base/rules/link-ids-for-the-human.md and the role texts that mention links: state the exact formats and that agents always use 'bridle link' (now for documents and specs too), never build URLs by hand.
3. docs/design/cli.md for the new 'bridle link' arguments; CHANGELOG.
Model: Sonnet. just check.
ORDER: land only after ui-judt has landed and the UI is installed (the orchestrator queues this then); old URLs keep working through ui-judt's redirects.

## Thread

### note · external:orchestrator · 2026-10-08T02:22:07.131Z
From orchestrator: br-enx3 ready (the human's go tonight, quotes in the body). Plan it, but queue it only when I say: it must land after bridle-ui's ui-judt is landed and installed. Then it goes ahead of everything except br-5p3z.
