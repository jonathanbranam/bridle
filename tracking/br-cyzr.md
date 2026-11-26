+++
id = "br-cyzr"
title = "Show the installed UI version and how far it is behind bridle-ui main in bridle status"
kind = "feature"
state = "claimed"
created_at = "2026-11-26T16:30:54.066Z"
updated_at = "2026-11-26T17:24:38.463591Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wcyzr",
]
ticket = "cyzr"
+++

docs/tickets/open/ui-version-status-cyzr.md

## Thread

### note · external:orchestrator · 2026-11-26T16:31:19.964Z
readied by external:orchestrator

### note · agent:wcyzr · 2026-11-26T17:24:38.463Z
Work committed on bridle/wcyzr; 'just check' is running now. Behind-main choice: count commits on main of this machine's bridle-ui clone (the bridle-ui entry in ~/.bridle/daemons/), no network; otherwise unknown with the reason. UI: Status.ui and gateway GET /api/v1/ui. npm run install-ui is unchanged and writes no bridle-version, so such installs show as 'installed another way' with version unknown.
