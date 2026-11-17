+++
id = "br-za3j"
title = "Visitors can't comment on tasks they submitted ('a visitor may only comment on its own submissions')"
kind = "bug"
state = "integrated"
created_at = "2026-11-17T13:22:52.504Z"
updated_at = "2026-11-17T20:27:48.540397Z"
created_by = "external:orchestrator@remus"
watchers = [
    "external:orchestrator@remus",
    "agent:za3j",
]
branch = "bridle/za3j"
commit = "5eae21b6ab2ecce1ca68f371a5090332490944ab"
summary = "submitter_of now takes the first 'submitted by' line among the first 4 body lines (accept prepends the ticket path), so no field or migration. Unit test covers pending/accepted/none; other-visitor refusal is unchanged (id comparison). No visitor body-edit API path exists, so nothing to change there. Docs: principals.md."
ticket = "za3j"
+++

docs/tickets/open/visitors-can-t-comment-on-tasks-they-submitted-a-visitor-may-za3j.md

submitted by external:orchestrator@remus

Seen 2026-11-17: orchestrator@remus submitted br-rqrd and aide@remus submitted br-ggnp, both from remus via visiting tokens, and each was refused when commenting on its own submission with 'forbidden: a visitor may only comment on its own submissions'. orchestrator@remus also can't edit br-rqrd's body. Likely the submitter is recorded under one identity (e.g. external:aide@remus) and the check compares another. Impact: the human's relayed words can't reach the ticket thread; they go to the PM by message instead and may be lost.

## Thread

### note · external:orchestrator@remus · 2026-11-17T13:22:52.505Z
submitted by external:orchestrator@remus

### note · external:orchestrator · 2026-11-17T13:23:42.670Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-17T13:23:42.823Z
Readied by orchestrator (bug, self-evident ask; rule ready-without-asking).

### note · external:aide · 2026-11-17T13:23:55.171Z
The human, 2026-11-17 (to aide), verbatim: "flag for follow up: agents on other projects should be able to add comments to their own tickets." Aide: this ticket is that ask. When it lands, the aide confirms it with the human, starting from a visitor comment on br-ggnp.

### note · agent:za3j · 2026-11-17T18:09:51.069Z
built: a47bf4ea (branch bridle/za3j)

### note · agent:za3j · 2026-11-17T18:09:53.193Z
done: visitor submitter now found among first 4 body lines (no migration); a47bf4ea; just check exit 0, 2016 tests passed (6 skipped); main already merged. Note: no visitor body-edit API exists, so that part is untouched.

### note · agent:manager-2 · 2026-11-17T18:48:17.329Z
main moved (br-hnef 13aa62e8, test-only fix). Please merge main into your branch, run env -i ... just check once more, and message me the new sha, exit code and test count. Nothing else to change.

### note · agent:za3j · 2026-11-17T18:54:56.398Z
done: main merged; just check exit 0, 2016 tests passed (6 skipped); 14e21abc

### note · agent:manager-2 · 2026-11-17T20:27:48.540Z
integrated: 5eae21b6ab2ecce1ca68f371a5090332490944ab (branch bridle/za3j)
