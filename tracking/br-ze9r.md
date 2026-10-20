+++
id = "br-ze9r"
title = "Role prompts say tickets settle ~5 minutes; the setting is 10 minutes"
kind = "fix"
state = "integrated"
created_at = "2026-10-18T15:11:16.988Z"
updated_at = "2026-10-20T04:33:15.629364Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wze9r",
]
branch = "bridle/wze9r"
commit = "a0f7903836b497ecd55548138583c158450ee565"
summary = """The two role prompts that said a ticket settles "~5 minutes" now say 10 minutes and name the setting: workflow/base/roles/manager.md (the settle bullet) and workflow/base/roles/project-manager.md (the planned-tickets bullet). Both list what restarts the period (creation, a human comment or edit, a relay of the human's words), as docs/design/agent-host/roles-and-config.md does. The invalid-settle warning in crates/bridle-daemon/src/config.rs said "using 5m"; the fallback is DEFAULT_SETTLE (10m), so it now says 10m. No default changed. A grep of workflow/ and docs/ found no other current settle wording at 5 minutes; the remaining hits are resolved tickets, role-notes history and the open zta7 ticket, which describes the 5m state it is about to change. Not changed: the "5m" example durations in config.rs and settle tests, and a test comment in crates/bridle-daemon/src/tasks.rs (~4086) that says "5 minute settle period" for a test that sets 5m explicitly."""
ticket = "ze9r"
+++

docs/tickets/open/role-prompts-say-tickets-settle-5-minutes-the-setting-is-10-ze9r.md

## Thread

### note · agent:pm-1 · 2026-10-18T15:11:58.641Z
Brief (pm-1). Model: Haiku. Edit the two lines named in the ask (workflow/base/roles/project-manager.md and manager.md) plus any other settle wording the grep finds. Acceptance: just check. Out of scope: changing the default.

### note · agent:wze9r · 2026-10-20T04:32:59.051Z
built: dbf4b36667cf491fe4420f888a8841e7a626a574 (branch bridle/wze9r)

### note · agent:wze9r · 2026-10-20T04:33:02.019Z
done: settle wording in both role prompts now says 10 minutes and names [tasks] settle (restart triggers listed); config.rs invalid-settle warning says 10m; CHANGELOG Fixed line; just check exit 0 (1925 run, 0 failed); tip dbf4b36667cf491fe4420f888a8841e7a626a574

### note · agent:manager-2 · 2026-10-20T04:33:10.870Z
integrated: a0f7903836b497ecd55548138583c158450ee565 (branch bridle/wze9r)

### note · agent:manager-2 · 2026-10-20T04:33:15.629Z
cleanup: removed nothing; kept agent wze9r (background job pid 82252)
