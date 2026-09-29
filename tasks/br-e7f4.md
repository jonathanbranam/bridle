+++
id = "br-e7f4"
title = "Stop CHANGELOG.md merge conflicts and committed conflict markers"
kind = "bug"
state = "open"
created_at = "2026-09-29T01:24:33.268Z"
updated_at = "2026-09-29T01:24:33.268Z"
+++

Goal: the manager committed CHANGELOG.md conflict markers three times on 2026-09-28 when merging worker branches that each add a line. Do: (1) add .gitattributes with 'CHANGELOG.md merge=union' at the repo root, so parallel appends merge cleanly; (2) in the manager skill or role under workflow/base/ (find where the merge steps are: workflow/base/roles/manager.md and skills), add one step before committing or pushing a merge: run git grep -nE '^(<<<<<<<|>>>>>>>) ' on the tree and refuse to push if anything matches. Also check the union driver works in a worktree merge by a quick test of two branches appending to CHANGELOG.md (a shell-free Rust test using git in a temp repo is fine if the repo has such helpers, otherwise verify by hand and say so in the handoff). Acceptance: just check passes. Model: Haiku. Out of scope: reworking how the CHANGELOG is written (br-1f62), other files.
