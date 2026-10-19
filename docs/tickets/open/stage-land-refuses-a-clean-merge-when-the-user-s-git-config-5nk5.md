---
id: br-5nk5
title: stage-land refuses a clean merge when the user's git config says merge.ff=only
kind: bug
created: 2026-10-19
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-upxq, br-2ae8, br-ewk8, br-tj7q]
---

## The ask

Found in the human's hands-on try (br-ewk8, ticket br-2ae8). designer-2ae8-design's first `stage-done` failed with "Not possible to fast-forward" because its branch was behind main. The designer rebased its own branch by hand and re-ran it, and then the stage landed (e8940620).

The human, 2026-10-19, to advisor tj7q: "How does the manager handle this? If a doc-only change is a clean merge, why rebase?"

Cause: `stage_land::land` (`crates/bridle-daemon/src/stage_land.rs`) runs a plain `git merge --no-edit` in the project checkout. The human's `~/.gitconfig` (and `~/.gitconfig.local`) set `merge.ff=only`, so any merge that is not a fast-forward is refused, even a clean one. The manager's landing (`integrator::land`) already guards against this: it runs `git -c merge.ff=true merge --squash`, with the comment "`merge.ff=only` in a user's config would make a squash of a moved base fail". stage-land (br-upxq) missed that guard.

The fix: stage-land's merge ignores the user's `merge.ff` (for example with `-c merge.ff=true`, as the integrator does), so a clean merge of a stage branch that is behind main lands without a rebase. A real conflict still fails back to the writer, as now. Add a test where the branch is behind main and `merge.ff=only` is set in the repo config.
