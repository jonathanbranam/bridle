---
id: br-t6ev
title: "Stages: write every unblocked stage before asking for review (design and specs together, one writer in turn)"
kind: fix
created: 2026-10-19
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-2ae8, br-ewk8, br-tj7q, br-ewhy]
---

## The ask

Found in the human's hands-on try (br-ewk8, ticket br-2ae8). After the proposal was signed, design and specs (both `requires: [proposal]`) were written one after the other with a review wait between them: designer-2ae8-design wrote design; `stage-done` set `claimed_by: human` (waiting for the signature); the stage watch gives no writer to a claimed ticket, so specs started only after the human signed the design.

The human, 2026-10-19, to advisor tj7q:

> specs and design both only depend on proposal. That means both specs and design should have been written at the same time and then brought to me for review. It shouldn't be sequential there, because once the proposal's written, both specs and design can be written.

> I think what should happen is the same agent should do both of them, one after another, not do one and then mark it. [...] I don't see any reason to necessarily have two agents, as long as the instructions are being given to the agent sequentially: fully do the specs, then fully do the design. It shouldn't matter which comes first.

> Do whatever's easy. Whatever's easy today, we can just run with that.

> If I finish the proposal and approve it, then agents should write both specs and design. They should come to me, and I can handle them independently, right? I should be able to accept the specs and rework the design somehow.

Cause: `crates/bridle-daemon/src/stage_watch.rs` keeps one writer per ticket and gives it the first unblocked stage; `stage-done` (`crates/bridle/src/stage_cmds.rs`) writes `claimed_by: human` as soon as any stage waits for a signature, and the watch skips a claimed ticket, so other unblocked stages wait for that signature.

The ask (the easy way, for today):
- One designer writes every stage that is unblocked, one after another, each fully (written, committed, `stage-done`, landed on main) before the next. Its prompt says so: after each `stage-done`, run `ticket next` again and write the next unblocked stage; end when none is left. Order doesn't matter.
- A signature wait no longer stops the other unblocked stages: `claimed_by: human` is written only when no stage is left unblocked (or the watch ignores the human's signature claim while a stage is unblocked; pick the simpler).
- The human then gets both stages for review together and handles each on its own: `approve` one, `changes` the other. `tasks` (requires both) waits for both signatures, as now.
- Each stage is written from its own inputs (`ticket next`'s `depends_on`), not from the sibling stage: the prompt tells the writer not to let one shape the other. Real independence (separate writers) is the follow-up ticket.
- Update `docs/design/stages.md` ("Writers and the timer", "Marks, signing and claims").
