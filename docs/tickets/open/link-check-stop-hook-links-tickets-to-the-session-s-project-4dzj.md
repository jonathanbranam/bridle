---
id: br-4dzj
title: link-check Stop hook links tickets to the session's project, not the ticket's (wrong links from a home-project orchestrator)
kind: bug
created: 2026-10-18
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The `bridle hook link-check` Stop hook links a ticket id to the session's own project, not the project that owns the ticket. A home-project orchestrator (e.g. dalek-home, br-7ytg) naming bridle tickets gets wrong links such as /p/dalek-home/tickets/da-gb97; the right one is `bridle link br-gb97 --project bridle`.

Fix: resolve an id to its owning project. Honour a qualified id's prefix (br-, da-). For a bare id, prefer the project where it exists, falling back to the session's project. Files: the link-check hook in crates/bridle (grep `link-check`), rule `link-ids-for-the-human`.

Verify: unit tests for a qualified id of another project, a bare id existing only in another project, and a bare id in the session's project; `just check` passes.

Model: Sonnet. Out of scope: changing `bridle link` output format.
