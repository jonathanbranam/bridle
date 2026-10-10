+++
id = "br-k3jk"
title = "bridle-ui ships with bridle releases: design pass (designer): options and a recommendation in ticket t3dr"
kind = "research"
state = "integrated"
created_at = "2026-10-10T17:21:41.852Z"
updated_at = "2026-10-10T21:07:47.300583Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
size = "M"
branch = "bridle/dk3jk"
commit = "1fd04992f488d785411e1a3ebeca8e11a64684b2"
summary = "Wrote '## Design' into t3dr. Options: A bundle a pinned bridle-ui build in the release tarball (A1) or as a second asset (A2); B bridle-ui publishes its own release; C bridle shallow-clones and builds with Node on each machine; D embed in the binary; E nothing. Recommend A1: ui.pin in bridle git, release.yml builds it, daemon extracts ui/ after the binary swap (release mode only), fv86 installer extracts it too; the dev machine (tc7t) stays on its clone and install-ui. Four questions for the human are at the end. Note: ticket check flags tasks br-k3jk as not made from the ticket (task-filing, not my edit)."
+++

Role: designer (workflow/base/roles/designer.md). Ticket: docs/tickets/open/bridle-ui-ships-with-each-bridle-release-fixed-to-that-versi-t3dr.md. Read all of it, especially '## The human's requirements for the short term' (must: no hand-made bridle-ui clone set up with bridle; automated; tied to the bridle version. May: a separate command, tarball or download; a shallow or read-only clone bridle makes itself; a local Node build. Long term: part of the bridle install). Also read tc7t (br-785a, the dev machine following landings), fv86 (install from a release with no clone), chvf (binary and workflow move together by release), and docs/design/human-web-ui.md (the gateway serves ~/.bridle/ui/ with a version check).

Write '## Design' into the ticket: options for where the UI build comes from and how it is pinned to a bridle version, how it gets into ~/.bridle/ui/ on install and on self-upgrade, how the dev machine (tc7t) fits, trade-offs, a recommendation and questions for the human. Build nothing. Report on the task; the PdM sends the design to the human.

## Thread

### note · external:advisor/product-manager · 2026-10-10T17:21:48.539Z
advisor/product-manager (PdM): design pass for epic ui-releases (rank 5), with the human's short-term requirements now in ticket t3dr. Docs only. Tier it after machine setup and the current messaging work; a designer slot when free. br-t3dr (build) stays pending until the human reviews the design.

### note · agent:pm-1 · 2026-10-10T17:21:52.965Z
pm-1: model Sonnet. Docs-only: edit the ticket (a Design section) and report; no code. Migration: none (design).

### note · agent:dk3jk · 2026-10-10T21:07:39.254Z
done: t3dr Design written (recommend A1: UI bundled in release tarball, pinned by ui.pin); 0bcd6cfc

### note · agent:manager-2 · 2026-10-10T21:07:44.770Z
integrated: 1fd04992f488d785411e1a3ebeca8e11a64684b2 (branch bridle/dk3jk)

### note · agent:manager-2 · 2026-10-10T21:07:47.300Z
cleanup: removed agent dk3jk, branch bridle/dk3jk
