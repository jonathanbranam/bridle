+++
id = "br-mhjn"
title = "Bugs, fixes and self-evident tickets are readied without asking the human; the human can always say stop"
kind = "chore"
state = "integrated"
created_at = "2026-10-17T15:21:25.837Z"
updated_at = "2026-10-17T18:30:07.492737Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-17T15:21:28.894708Z"
branch = "bridle/wmhjn"
commit = "d1e007ff6993cb740740bee703b7837b1d15ab80"
summary = "New rule workflow/base/rules/ready-without-asking.md (must; orchestrator, PdM, advisor, aide, project-manager) says who readies what; 'ready only with the human's approval' wording replaced in the orchestrator, manager, advisor and project-manager roles, the tickets rule, cli.md, docs/cli, and ticket ready help. 'ticket new' without a body now warns on stderr that the ticket cannot be readied until the ask is under '## The ask' (no_ask_warning, unit-tested). CHANGELOG added. Left alone: orchestrator.md line on relayed approval (a different point); aide.md had no ready wording; historical docs/notes unchanged. 'hold: design' front matter is only documented in the rule, not enforced by 'ticket ready' (not asked for)."
ticket = "mhjn"
+++

docs/tickets/open/bugs-fixes-and-self-evident-tickets-are-readied-without-aski-mhjn.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T15:21:28.894Z
priority: normal -> high

### note · agent:wmhjn · 2026-10-17T17:54:37.007Z
done: rule ready-without-asking + ticket-new warning; just check exit 0, 1824 tests passed (last full 1823), main already merged; 74af3aef

### note · agent:manager-2 · 2026-10-17T18:29:53.047Z
integrated: d1e007ff6993cb740740bee703b7837b1d15ab80 (branch bridle/wmhjn)

### note · agent:manager-2 · 2026-10-17T18:30:07.492Z
cleanup: removed agent wmhjn, branch bridle/wmhjn
