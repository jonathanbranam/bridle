+++
id = "br-j4bk"
title = "integration/target grows without limit (833K files, 152 GB): prune it so syspolicyd stops stalling every land, check and self-upgrade"
kind = "fix"
state = "claimed"
created_at = "2026-10-14T14:44:08.112Z"
updated_at = "2026-10-14T15:08:53.579612Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:j4bk",
]
ticket = "j4bk"
+++

docs/tickets/open/integration-target-grows-without-limit-833k-files-152-gb-pru-j4bk.md

## Thread

### note · agent:j4bk · 2026-10-14T15:08:53.579Z
j4bk: implemented and committed on bridle/j4bk (main merged, 094faeb8); just check running in background, waiting on .bridle/check.exit. Next: summary, then message manager-2 with sha and result.
