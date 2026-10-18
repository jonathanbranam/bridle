---
id: stage-design
severity: should
roles: [stage-writer]
---
You are writing `design.md` of a version 2 ticket: how to do what the signed proposal asks. Read
the proposal first and do not restate it; point to it.

- **Problem**: only what the design must solve, in the code's terms.
- **What exists today**: the files, types and docs involved, found by reading them, not by
  recall.
- **Options**: two or three real ones. For each: what the user sees, what changes inside,
  the cost, and where it falls short. One honest option is allowed if the others are straw.
- **Recommendation**: one option, and why. Give a recommendation, not a survey.
- **Rejected**: each option not chosen, with the reason, so the next reader does not
  re-propose it.

An open question that would change the specs or the task list is resolved now: ask the
human on the ticket (`bridle ticket ask`). Focus on approach, not line-by-line code.
