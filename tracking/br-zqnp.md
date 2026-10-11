+++
id = "br-zqnp"
title = "bridle link and agent instructions emit the ticket page URL /p/{project}/tickets/{id}"
kind = "feature"
state = "integrated"
created_at = "2026-10-11T13:51:22.503Z"
updated_at = "2026-10-11T20:53:11.838781Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wzqnp",
]
branch = "bridle/wzqnp"
commit = "ea3091de7a64acf4d2c4be9decdd73d9f172abab"
summary = "bridle link now prints <public_url>/p/<project>/tickets/<id> (ticket, qualified or bare), /p/<project>/docs?path=<path> (repo-relative document path) and /p/<project>/specs?path=<file>#<id> (r-/s- ID, file found via bridle-docs). All need the project (project::resolve). Link rule, advisor/aide/orchestrator role text, cli.md and CHANGELOG updated; CLAUDE.md has no Link formats section. Old URLs redirect in the UI, base rule/role text reaches projects via workflow sync. Supersedes br-enx3 (not closed here)."
ticket = "zqnp"
+++

Ticket: docs/tickets/open/bridle-link-and-agent-instructions-emit-the-ticket-page-url-zqnp.md (read it; human approved 2026-10-11). This SUPERSEDES br-enx3 (planned for the old /tasks/ URLs): its remaining scope is folded in here.

Read bridle-ui's docs/design/url-scheme.md (Final formats, section 5; at /Volumes/Data/work/bridle-ui-workspace/bridle-ui if not in this repo) for the exact formats before changing anything.
Build: (1) crates/bridle/src/link.rs: `bridle link <id>` prints {public_url}/p/{project}/tickets/{id} (the one ticket page); silent when public_url is unset, as today. A ticket needs its project (project::resolve, as the ticket form does). (2) From enx3: `bridle link` also takes a repo-relative document path (e.g. docs/design/cli.md) and a spec ID and prints the document and spec forms from url-scheme.md. Tests per form. (3) The link rule: workflow/base/rules/link-ids-for-the-human.md and every role/rule/skill text and CLAUDE.md 'Link formats' that cites formats state the exact new formats and that agents always use `bridle link`, never build URLs by hand. (4) docs/design/cli.md for the new arguments; CHANGELOG.
Migration: base rule and role text reach projects through workflow sync; old URLs redirect in the UI, so nothing breaks; state it. Acceptance: just check passes; `bridle link br-xxxx` prints the new form; tests for ticket, document and spec forms. Model: Sonnet (the document and spec forms make it more than Haiku). Out of scope: bridle-ui.

## Thread

### note · external:advisor/product-manager · 2026-10-11T17:18:05.444Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve as recommended". Readied. zqnp supersedes br-enx3 (approved 10-08, planned for the old /tasks/ URLs): pm-1, fold enx3's remaining scope (document and spec links, the link rule with exact formats) into zqnp's plan, then drop enx3 as superseded.

### note · agent:wzqnp · 2026-10-11T20:53:03.848Z
done: bridle link emits ticket/docs/specs page URLs; just check exit 0, 1545 tests passed, 81bfdea9

### note · agent:manager-2 · 2026-10-11T20:53:08.971Z
integrated: ea3091de7a64acf4d2c4be9decdd73d9f172abab (branch bridle/wzqnp)

### note · agent:manager-2 · 2026-10-11T20:53:11.582Z
cleanup: removed nothing; kept agent wzqnp (background job pid 17027)

### note · agent:wzqnp · 2026-10-11T20:53:11.838Z
Built on branch bridle/wzqnp, commit 81bfdea9 (just check exit 0, 1545 tests). The installed bridle has no 'ticket built' subcommand (error: unrecognized subcommand 'built'), so it was not marked built; manager please mark/land.
