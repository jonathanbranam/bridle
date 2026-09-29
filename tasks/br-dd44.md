+++
id = "br-dd44"
title = "P4: suspect links: bridle trace suspect and confirm"
kind = "feature"
state = "planned"
created_at = "2026-09-29T05:50:50.369Z"
updated_at = "2026-09-29T05:50:54.752466Z"
+++

Goal (docs/design/traceability.md): a link whose recorded @hash differs from the upstream element's current hash is suspect. `bridle trace suspect` lists them (link, upstream id, recorded vs current hash), exit 1 if any (so a project can use it in checks); `bridle trace confirm <id>` rewrites that element's link hashes to current, line-edit only, rest of file byte-for-byte, like `spec id`. Uses the trace module from the trace-links task. Files: crates/bridle-spec/trace.rs, crates/bridle cli. Acceptance: just check passes; tests: edit upstream text -> suspect; confirm clears; file otherwise unchanged. Model: Sonnet. Out of scope: opening re-evaluate tasks on arch-revision (needs the arch-revision flow; separate). Runs after the trace-links task.
