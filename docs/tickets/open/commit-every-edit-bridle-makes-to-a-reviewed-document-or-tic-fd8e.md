---
id: fd8e
title: Commit every edit bridle makes to a reviewed document or ticket (status markers, thread IDs, frontmatter), so the clone is never left dirty
kind: bug
opened: 2026-10-10
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [4cgx, p896, x8jt, m3wg, v8kn]
tasks: [br-fd8e]
---

## The ask

The human, 2026-10-10 ~5:15 PM ET, verbatim (to the aide):

> There's a gap I noticed earlier: when the doc reviewer updates the doc with comments or marks it
> red or anything, that file needs to be committed. We should be pushing regularly, so I don't know
> if he needs push permissions, but he definitely needs to commit the file

## Facts (2026-10-10, the aide)

- The main clone holds two uncommitted ticket edits that no agent owns:
  - `4cgx`: every `[sent 2026-10-09 22:44 EDT]` marker on the human's comments turned into
    `[read 2026-10-09 22:45 EDT]`. Per `docs/design/human-web-ui.md` (review section) the
    **daemon** rewrites `[sent]` to `[read]` once the agent has read the message, and gives a
    hand-typed thread its `c<n>` ID. Neither write is committed.
  - `p896`: `tasks: []` became `tasks: [br-p896]` (a task filed from the ticket wrote the
    frontmatter and didn't commit it).
- What is committed: the human's comments (`review: human comments on ...` commits, af2a0e95,
  ec5e53c2) and the document-reviewer's rounds (role step 5, "Commit each round"; 98eecbcf).
  So the gap is the writes made by bridle itself, not the reviewer's own edits.
- Cost: the orchestrator had to stash these files to run the benchmark preflight (br-g9xe,
  2026-10-10 17:08Z), and an uncommitted edit has blocked a landing before (m-2035, ticket v8kn).
  Related: m3wg (sessions sharing the working copy).

## The ask, itemised

1. Every edit bridle makes to a document or ticket in the clone (review status markers, thread
   IDs, red/highlight marks, frontmatter written by `ticket task` and the like) is committed when
   it is made, so the clone is never left dirty by bridle.
2. Pushing: the human expects regular pushes and is unsure whether the reviewer needs push
   permission. Rule `one-pusher-for-the-integration-branch` allows only the owner clone to push
   main; the design should say whether these commits ride the normal push or need anything new.
