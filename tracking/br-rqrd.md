+++
id = "br-rqrd"
title = "Base rule: skip a ticket's settle period only when the human explicitly says so"
kind = "docs"
state = "open"
created_at = "2026-11-17T12:14:25.505Z"
updated_at = "2026-11-17T12:14:52.837889Z"
created_by = "external:orchestrator@remus"
watchers = ["external:orchestrator@remus"]
ticket = "rqrd"
+++

docs/tickets/open/base-rule-skip-a-ticket-s-settle-period-only-when-the-human-rqrd.md

submitted by external:orchestrator@remus

## The ask

Add a base workflow rule (workflow/base/rules/, binds every project), e.g. `settle-period`, and align `bridle ticket skip-settle --help` and the orchestrator role text with it:

Don't skip a ticket's settle period (the ~5-minute cooldown) unless the human explicitly says to skip it. A deadline, an urgent priority or "you have an hour" is not an instruction to skip. One exception: in bridle's own project, an agent may skip it for an urgent, critical fix it has determined itself (e.g. a red main or a downtime fix).

The human's words (2026-11-17, via advisor/flume, verbatim): "Don't skip the 5-minute cooldown. Plus, you built that in 3 minutes, so it was completely unnecessary at this point. Don't skip the cooldown unless explicitly told to skip the cooldown. Or, in the case of bridle, if there are urgent, critical fixes determined by the agent, it can skip the cooldown. Otherwise, don't skip it." The human asked for a standing base rule, not just a note.

Context: orchestrator@remus skipped settle on fl-macf (flume), citing "You have one hour before I get to work". The ticket was built and landed in about 3 minutes, so the skip saved nothing.

Today, skip-settle's --reason help says the orchestrator and PM may skip "because the human asked or for an urgent downtime fix". Tighten it to "the human explicitly said to skip it", plus the bridle-only critical-fix exception.

Kind: docs (light; Haiku). Verify: `bridle rules explain settle-period` shows the base rule.

## Thread

### note · external:orchestrator@remus · 2026-11-17T12:14:25.506Z
submitted by external:orchestrator@remus

### note · external:orchestrator · 2026-11-17T12:14:52.837Z
readied by external:orchestrator
