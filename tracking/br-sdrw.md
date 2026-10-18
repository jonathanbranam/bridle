+++
id = "br-sdrw"
title = "Turn off Claude Code prompt suggestions in every bridle session"
kind = "chore"
state = "built"
created_at = "2026-10-07T02:17:09.411Z"
updated_at = "2026-10-18T08:18:27.254458Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
    "agent:wsdrw",
]
branch = "bridle/wsdrw"
commit = "c3a528ca137dbcbd9fef7ed00b0e45fe469fdc3a"
summary = "promptSuggestionEnabled:false added to the spawned-agent --settings (bridle-claude command.rs), to the LEAN settings shared by orchestrator/advisor/aide sessions (bridle session.rs), and to the project .claude/settings.json. Tests assert it in both. CHANGELOG line added."
ticket = "sdrw"
+++

docs/tickets/open/turn-off-claude-code-prompt-suggestions-in-every-bridle-sess-sdrw.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:39.816Z
watching the task

### note · external:advisor/product-manager · 2026-10-17T15:26:07.672Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wsdrw · 2026-10-18T08:18:26.926Z
built: c3a528ca137dbcbd9fef7ed00b0e45fe469fdc3a (branch bridle/wsdrw)

### note · agent:wsdrw · 2026-10-18T08:18:27.254Z
done: promptSuggestionEnabled=false in agent and session --settings plus .claude/settings.json, tested; just check exit 0, 1863 tests; c3a528ca137dbcbd9fef7ed00b0e45fe469fdc3a
