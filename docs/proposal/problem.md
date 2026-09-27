# The problem

The current system is a good *workflow* on a poor *substrate*:

| Hurts today | Why |
|---|---|
| An agent cannot wait on another agent | There is no dependency edge and no wake-up; the driver holds "wait for the test-config fix" in its head |
| Agents cannot talk to each other | A subagent's only channel is its final report, to its parent |
| Two tasks cannot touch the same spec at once | OpenSpec folds deltas only at archive, matches requirements by heading text, and so refuses the second change (the *same-spec pile-up*, research 10 §2b) |
| The workflow is sequential with many human checks | Plan approval, blocking questions, archive, and the pile-up rule all wait on one person |
| Every project carries its own copy of the workflow | Six repos, six sets of ~10 OpenSpec skills, six CLAUDE.md files restating overlapping rules with no precedence |
| Gherkin generation is slow and brittle | Generated `.feature` files are committed, so they need a staleness check, two validation layers, `gherkin-official` and `uv run` |

The workflow's *shape* is right and has independent confirmation: plan with a
strong model, implement with a cheaper one, review with a strong one that is not
the implementer (Wheelhouse converged on the same three steps; research 06 §2).
What changes is the machinery under it and how much of it waits on the human.
