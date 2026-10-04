+++
id = "br-tgdn"
title = "Landing refuses a clone with unrelated uncommitted edits; document review (x8jt) now leaves them routinely"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T03:11:21.857Z"
updated_at = "2026-10-04T03:30:35.787063Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
branch = "bridle/land-dirty"
commit = "bc8c745526f69dca2856bd3b0135c6d255a7f46c"
summary = "advance() in integrator.rs now compares the integration checkout's dirty files with 'git diff --name-only old new' and refuses only on an overlap, naming the files; unrelated dirty files stay (git ff-merges past them). Tests in land_test.rs (overlap refused, unrelated lands); docs and CHANGELOG updated. The 'main moved' error is a different check (tip != old, a real race), so docs-only moves were not changed."
+++

Bug, size S. Found 2026-10-04 03:06Z: manager-2's landing of br-qttb and br-e35h was blocked by one uncommitted ticket edit in the bridle clone (m-4297). With x8jt live, the clone's main checkout often holds uncommitted edits (the human's comments waiting out the ~7 min quiet period; br-qttb's '· sent' marks until the document agent commits), so each would block every landing meanwhile.
Fix, smallest first: landing goes ahead when the uncommitted files aren't touched by the merge (git merges past unrelated dirty files) and refuses only on an overlap, naming the overlapping files. Where: the landing/integrate code, start at crates/bridle-daemon/src/integrator.rs (grep 'uncommitted'; worktree.rs and supervisor.rs also mention it). Related: landings also fail with 'main moved' on docs-only commits; if that is the same check, a docs-only move needn't force a re-merge (look, and fix only if it is the same check; otherwise say so on the task).
Docs: docs/design/agent-host/ (the landing section) in step, CHANGELOG. Tests: dirty unrelated file => lands, dirty file the merge touches => refused with its name, clean => as before. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: changing who commits the document edits.

## Thread

### note · agent:land-dirty · 2026-10-04T03:28:41.866Z
'main moved' is a different check (checkout HEAD != the tip the landing started from, a genuine race), not the dirty check; left unchanged.

### note · agent:land-dirty · 2026-10-04T03:28:41.889Z
done: landing refuses a dirty integration checkout only when the merge touches the dirty files (names them); check green; d54ba8da. 'main moved' is a separate check, unchanged (see comment).

### note · agent:manager-2 · 2026-10-04T03:28:47.899Z
integrated: bc8c745526f69dca2856bd3b0135c6d255a7f46c (branch bridle/land-dirty)

### note · agent:manager-2 · 2026-10-04T03:30:35.787Z
cleanup: removed agent land-dirty, branch bridle/land-dirty
