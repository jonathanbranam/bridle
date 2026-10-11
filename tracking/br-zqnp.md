+++
id = "br-zqnp"
title = "bridle link and agent instructions emit the ticket page URL /p/{project}/tickets/{id}"
kind = "feature"
state = "claimed"
created_at = "2026-10-11T13:51:22.503Z"
updated_at = "2026-10-11T20:33:38.206342Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wzqnp",
]
ticket = "zqnp"
+++

Ticket: docs/tickets/open/bridle-link-and-agent-instructions-emit-the-ticket-page-url-zqnp.md (read it; human approved 2026-10-11). This SUPERSEDES br-enx3 (planned for the old /tasks/ URLs): its remaining scope is folded in here.

Read bridle-ui's docs/design/url-scheme.md (Final formats, section 5; at /Volumes/Data/work/bridle-ui-workspace/bridle-ui if not in this repo) for the exact formats before changing anything.
Build: (1) crates/bridle/src/link.rs: `bridle link <id>` prints {public_url}/p/{project}/tickets/{id} (the one ticket page); silent when public_url is unset, as today. A ticket needs its project (project::resolve, as the ticket form does). (2) From enx3: `bridle link` also takes a repo-relative document path (e.g. docs/design/cli.md) and a spec ID and prints the document and spec forms from url-scheme.md. Tests per form. (3) The link rule: workflow/base/rules/link-ids-for-the-human.md and every role/rule/skill text and CLAUDE.md 'Link formats' that cites formats state the exact new formats and that agents always use `bridle link`, never build URLs by hand. (4) docs/design/cli.md for the new arguments; CHANGELOG.
Migration: base rule and role text reach projects through workflow sync; old URLs redirect in the UI, so nothing breaks; state it. Acceptance: just check passes; `bridle link br-xxxx` prints the new form; tests for ticket, document and spec forms. Model: Sonnet (the document and spec forms make it more than Haiku). Out of scope: bridle-ui.

## Thread

### note · external:advisor/product-manager · 2026-10-11T17:18:05.444Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve as recommended". Readied. zqnp supersedes br-enx3 (approved 10-08, planned for the old /tasks/ URLs): pm-1, fold enx3's remaining scope (document and spec links, the link rule with exact formats) into zqnp's plan, then drop enx3 as superseded.
