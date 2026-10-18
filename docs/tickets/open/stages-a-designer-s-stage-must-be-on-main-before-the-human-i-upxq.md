---
id: br-upxq
title: "Stages: a designer's stage must be on main before the human is asked to review it"
kind: bug
created: 2026-10-18
created_by: external:advisor/tj7q
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ewk8, br-2ae8, br-ewhy, br-tj7q]
---

## The ask

Found in the human's first version 2 try (br-ewk8, ticket br-2ae8). The human, 2026-10-18, to advisor tj7q, verbatim:

> OK this is a failure. that is not how this can work. I'm not checking a branch to review a proposal. It has to be on main for me to review it.

And: "this needs to be filed and fixed with a real solution somehow." So the fix is in the stage flow itself, not a manager remembering to land designer branches.

## What happened

- The daemon spawned `designer-2ae8-proposal` (stage watch, br-ewhy). It wrote the proposal, ran `ticket stage-done br-2ae8 proposal`, committed 9ee0bc1f on `bridle/designer-2ae8-proposal`, messaged manager-2 and ended.
- The advisor was told "stage proposal written, waiting for the human's signature". But main, and so the web UI and `ticket show`, still had the empty template. manager-2 sat idle with the report; nothing landed the branch.
- docs/design/stages.md already says a stage counts as done for the daemon only once it's on the project checkout, "so the branch must land first", but nothing makes that happen, and the "waiting for signature" note goes out before it.
- Smaller: the designer's spawn prompt says "tell your manager with `bridle send`" without a name; `bridle send manager` failed and the designer had to look up manager-2 itself.

## Decision (the human, 2026-10-18)

Chosen with advisor tj7q from three options (rejected: designers commit straight to main, which shares the human's checkout and drops isolation; the manager lands designer branches, which is what failed). The human: "agree with A" and, on the steps below, "agree, yes, approved and ready to work". The human's own outline, verbatim:

> 1. agents work on worktrees; they shouldn't build bridle or any project, just work on documents on the branch
> 2. there is an existing command the agent uses to say "this stage is done"
> 3. the daemon receives that command, verifies the only chnages are to the ticket the agent is working on
> 4. the daemon merges the worktree branch to main
> 5. any failures in 3-4 are sent where? manager?
> 6. then the daemon updates the stage as per existing rules (move to next stage, send to human, etc)

The design, as agreed:

1. The designer works in its own worktree and branch and writes documents only (the ticket file and its stage folder). It builds nothing.
2. The designer commits first, then runs `bridle ticket stage-done <id> <stage>`. Today `stage-done` runs only in the designer's checkout: it marks `written` in the frontmatter and posts the "waiting for the human's signature" note before anything is committed (crates/bridle/src/stage_cmds.rs `stage_done`). That changes: `stage-done` asks the daemon to verify the commit and land it.
3. The daemon checks that the branch's changes (against main) touch only this ticket's file and its stage folder, and that main's checkout has no uncommitted edits to those paths.
4. The daemon merges the designer's branch into main.
5. Failures:
   - First, back to the designer: `stage-done` fails at once with the reason (for example "touches crates/foo.rs" or "conflicts with main"). The designer fixes its branch and runs it again.
   - If the designer can't fix it, or ends without a successful `stage-done`: a note on the ticket's thread and one message to `external:advisor/product-manager`, the same route as stage timeouts (br-ewhy).
   - Not to the manager.
6. Only after the merge does the daemon record the stage as written and apply the existing rules: the next stage's writer, or the signature wait and its note, now naming the commit on main.

## Done when

- A stage a designer writes is on main before anyone is asked to review or sign it, for every stage, not only `review: human` ones, by the decision above.
- The designer role (workflow/base/roles/designer.md), the spawn prompt (stage_watch.rs `writer_prompt`) and docs/design/stages.md say: commit, then `stage-done`; no manager step.
- The "waiting for the human's signature" note (and any message to an advisor or the human) goes out only once the stage is on main, and names the main commit.
- The designer's prompt names its manager (or the land step needs none).
- A test: a designer stage with `review: human` is readable on main (`ticket show`) when the signature request is sent.
- Tests: a branch touching a file outside the ticket and its stage folder is refused with the path named; a merge conflict refuses `stage-done`, and a designer ending without one alerts the product manager once.
- br-2ae8's proposal (9ee0bc1f) is on main so the try can continue (done by hand now; this ticket is for the next one).
