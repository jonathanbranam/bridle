+++
id = "br-dd44"
title = "P4: suspect links: bridle trace suspect and confirm"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T05:50:50.369Z"
updated_at = "2026-09-29T06:33:00.126821Z"
branch = "bridle/suspect-links"
commit = "32ad18d691e99342f275d7169e05fbf63d0124d3"
summary = "Added Graph::suspects and rewrite_links (crates/bridle-spec/src/trace.rs) plus 'bridle trace suspect' (exit 1 if any; --json) and 'bridle trace confirm <id>' (edits only the requirement's heading line). Unit test covers edit upstream -> suspect, confirm clears, rest of file unchanged. Docs and CHANGELOG updated."
+++

Goal (docs/design/traceability.md): a link whose recorded @hash differs from the upstream element's current hash is suspect. `bridle trace suspect` lists them (link, upstream id, recorded vs current hash), exit 1 if any (so a project can use it in checks); `bridle trace confirm <id>` rewrites that element's link hashes to current, line-edit only, rest of file byte-for-byte, like `spec id`. Uses the trace module from the trace-links task. Files: crates/bridle-spec/trace.rs, crates/bridle cli. Acceptance: just check passes; tests: edit upstream text -> suspect; confirm clears; file otherwise unchanged. Model: Sonnet. Out of scope: opening re-evaluate tasks on arch-revision (needs the arch-revision flow; separate). Runs after the trace-links task.

## Thread

### note · agent:manager-2 · 2026-09-29T06:33:00.126Z
integrated: 32ad18d691e99342f275d7169e05fbf63d0124d3 (branch bridle/suspect-links)
