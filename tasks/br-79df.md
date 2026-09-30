+++
id = "br-79df"
title = "Base rule: check and update the docs as the last step of every change (3ndf)"
kind = "chore"
state = "dropped"
created_at = "2026-09-30T23:27:11.171Z"
updated_at = "2026-09-30T23:43:15.756467Z"
size = "S"
summary = "Added CHANGELOG entry for docs-current rule (br-987b had implemented the rule and worker handoff; br-0f46 already dropped). Commit ef33f8c."
+++

Ticket: docs/tickets/open/docs-kept-current-as-part-of-every-change-3ndf.md (supersedes the stale imported task br-0f46; drop br-0f46 with a note when done).

Goal: a worker finishing a change checks the docs that describe the changed behaviour and updates them in the same branch, or says in its done report that none needed it.

Do:
- Add a base rule workflow/base/rules/docs-current.md (match the style and length of the neighbouring rules, e.g. missing-tools.md): at the end of a change, find docs that describe the changed behaviour (docs/design/**, cli.md, storage.md, README/docs index, product briefs if present), update them, and state in the done report "docs: updated X" or "docs: none needed".
- Make it a named step of the worker finish: edit workflow/base/roles/worker.md (replace the one-line Docs bullet with a pointer to the rule) and the worker finishing skill if one exists under workflow/base/ (grep for it).
- Keep the repo copy in step with any rendered copy: run whatever sync the repo uses for .bridle/rules if bridle keeps a checked-in rendering (check docs/design/workflow-layers.md; if unsure, ask the manager).
- Add a CHANGELOG.md entry on top.

Acceptance: just check passes. Model: Haiku.
Out of scope: a separate docs-checking agent; any code change; checking that the rule was followed.

## Thread

### note · agent:docs-rule · 2026-09-30T23:43:10.924Z
done: Added CHANGELOG entry for docs-current rule (implementation was in br-987b, br-0f46 already dropped); ef33f8c. First check had parses_fast flake, retry passed all 933 tests.

### note · agent:manager-2 · 2026-09-30T23:43:15.756Z
dropped: Already implemented by br-987b (0541c98: workflow/base/rules/docs-current.md and the worker finish step); the worker found nothing left but a CHANGELOG line.
