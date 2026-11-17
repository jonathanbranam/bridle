---
id: br-rqrd
title: "Tighten the existing settle-skip wording: only when the human explicitly says so (no new rule)"
kind: docs
created: 2026-11-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Add a base workflow rule that binds every project, and align the command help and the role text with it.

The human, 2026-11-17, via advisor/flume (relayed by orchestrator@remus), verbatim: "Don't skip the 5-minute cooldown. Plus, you built that in 3 minutes, so it was completely unnecessary at this point. Don't skip the cooldown unless explicitly told to skip the cooldown. Or, in the case of bridle, if there are urgent, critical fixes determined by the agent, it can skip the cooldown. Otherwise, don't skip it." The human asked for a standing base rule, not a note.

Context: orchestrator@remus skipped settle on flume's fl-macf citing "You have one hour before I get to work"; it was built and landed in about 3 minutes, so the skip saved nothing.

The rule (text for the worker to tighten, in the repo's rule format with a Why line carrying the quote above): do not skip a ticket's settle period unless the human explicitly says to skip it. A deadline, an urgent priority or "you have an hour" is not such an instruction. One exception, in bridle's own project only: an agent may skip it for an urgent, critical fix it has itself determined (a red main, a downtime fix).

## Files (the worker confirms)

- New rule `workflow/base/rules/settle-period.md` (look at a sibling such as workflow/base/rules/kiss.md or the other rules for frontmatter and the "must/should" level; this should be `must`).
- `bridle ticket skip-settle --help` text (crates/bridle/src/cli.rs, grep `skip-settle` and `--reason`): today it says the orchestrator and PM may skip "because the human asked or for an urgent downtime fix"; change to "the human explicitly said to skip it", plus the bridle-only critical-fix exception.
- The orchestrator and project-manager role text under workflow/ that mentions skipping settle (grep `skip-settle`, `settle`). The PM role line "Don't skip it unless the human asks or it's an urgent downtime fix" must match.
- Any doc that describes the settle skip (grep docs/design for skip-settle).

## Acceptance

`bridle rules explain settle-period` shows the base rule; the help and role texts agree with it and with each other; `just check` with BRIDLE_* env unset (rule and snapshot tests may need updating: if a test lists base rules, add the new one). No project-file change in projects: base rules reach them through the existing workflow sync, so no migration; say so in the CHANGELOG entry if one is made for rules. Model: Haiku (light, mechanical).
