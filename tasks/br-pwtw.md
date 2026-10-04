+++
id = "br-pwtw"
title = "br-5paw follow-up: gateway document PUT commits on main too"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T01:15:10.719Z"
updated_at = "2026-10-04T01:30:47.426971Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
branch = "bridle/doc-gateway2"
commit = "9d0f0f60b9ff5506c5a52cca4a02ce064b43fade"
summary = "Document PUT in bridle-gateway (documents.rs) now commits on whatever branch is checked out, including main/master/dev; only a detached HEAD is refused (403). Removed PROTECTED_BRANCHES, renamed the error ProtectedBranch to DetachedHead, rewrote the test, updated human-web-ui.md and the CHANGELOG line."
+++

Orchestrator direction (m-4206): the document write must commit on whatever branch is checked out, main included (the trial doc gtzx lives on main). Remove the main/master/dev refusal in crates/bridle-gateway/src/documents.rs; keep refusal for detached HEAD, the hash check, path-inside-project, single-file commit. Update its test, docs/design/human-web-ui.md and CHANGELOG line. just check must pass.

## Thread

### note · agent:doc-gateway2 · 2026-10-04T01:29:25.158Z
done: PUT commits on any branch, detached HEAD still refused; tests, docs, CHANGELOG updated; 9126a99

### note · agent:manager-2 · 2026-10-04T01:29:31.499Z
integrated: 9d0f0f60b9ff5506c5a52cca4a02ce064b43fade (branch bridle/doc-gateway2)

### note · agent:manager-2 · 2026-10-04T01:30:47.426Z
cleanup: removed agent doc-gateway2, branch bridle/doc-gateway2
