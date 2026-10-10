+++
id = "br-k3jk"
title = "bridle-ui ships with bridle releases: design pass (designer): options and a recommendation in ticket t3dr"
kind = "research"
state = "planned"
created_at = "2026-10-10T17:21:41.852Z"
updated_at = "2026-10-10T17:21:52.965529Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
size = "M"
+++

Role: designer (workflow/base/roles/designer.md). Ticket: docs/tickets/open/bridle-ui-ships-with-each-bridle-release-fixed-to-that-versi-t3dr.md. Read all of it, especially '## The human's requirements for the short term' (must: no hand-made bridle-ui clone set up with bridle; automated; tied to the bridle version. May: a separate command, tarball or download; a shallow or read-only clone bridle makes itself; a local Node build. Long term: part of the bridle install). Also read tc7t (br-785a, the dev machine following landings), fv86 (install from a release with no clone), chvf (binary and workflow move together by release), and docs/design/human-web-ui.md (the gateway serves ~/.bridle/ui/ with a version check).

Write '## Design' into the ticket: options for where the UI build comes from and how it is pinned to a bridle version, how it gets into ~/.bridle/ui/ on install and on self-upgrade, how the dev machine (tc7t) fits, trade-offs, a recommendation and questions for the human. Build nothing. Report on the task; the PdM sends the design to the human.

## Thread

### note · external:advisor/product-manager · 2026-10-10T17:21:48.539Z
advisor/product-manager (PdM): design pass for epic ui-releases (rank 5), with the human's short-term requirements now in ticket t3dr. Docs only. Tier it after machine setup and the current messaging work; a designer slot when free. br-t3dr (build) stays pending until the human reviews the design.

### note · agent:pm-1 · 2026-10-10T17:21:52.965Z
pm-1: model Sonnet. Docs-only: edit the ticket (a Design section) and report; no code. Migration: none (design).
