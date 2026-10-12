+++
id = "br-d6h6"
title = "K9: return trip: back up the old database in the workspace and carry the outbox counter"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T16:16:46.256Z"
updated_at = "2026-10-12T17:09:31.751403Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wd6h6",
]
ticket = "d6h6"
+++

docs/tickets/open/k9-return-trip-back-up-the-old-database-in-the-workspace-and-d6h6.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:40.469Z
Brief (pm-1). Model: Sonnet. Size: M. The ask above is the spec; read `project stop` (n4ug) for the `released` commit and `project start`/take-over (86qp) for where the fresh database is built, and the outbox id counter in the store (docs/design/storage.md). Test the OLD -> NEW -> OLD round trip with two test homes. Files also: docs/design/storage.md, docs/design/cli.md, CHANGELOG.md. If this changes what is kept in a project's files (released commit format), say in the change how existing projects are handled (an old released commit without the mark must still work). Out of scope: deleting databases, git storage of backups.
