+++
id = "br-prf4"
title = "Setup gaps (c, d): bridle init prints 'bridle project start' as the next step and its .gitignore covers bridle's generated .claude files"
kind = "fix"
state = "planned"
created_at = "2026-10-16T02:24:07.877Z"
updated_at = "2026-10-16T02:36:09.894151Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "prf4"
+++

docs/tickets/open/setup-gaps-c-d-bridle-init-prints-bridle-project-start-as-th-prf4.md

## Thread

### note · agent:pm-1 · 2026-10-16T02:24:19.285Z
Brief (pm-1). Model: Haiku. Size: S. In `bridle init` (git grep -n "bridle serve" crates/bridle/src/init*.rs; and where it writes .gitignore) change the printed next step to `bridle project start`, and add `.claude/skills/bridle-*/` and `.claude/.bridle-sync-hooks.json` to the .gitignore init writes (copy the exact lines from meta-notes' .gitignore pattern if available in the repo docs; else use these two). Only for new projects; init must not rewrite an existing .gitignore beyond what it does today (it appends missing lines? keep that behaviour: add the two lines the same way). Tests: init output names project start; .gitignore contains both lines; running init twice does not duplicate. CHANGELOG. Acceptance: just check.

### note · external:advisor/product-manager · 2026-10-16T02:24:58.092Z
Readied by advisor product-manager under the human's go on dffz (2026-10-15 ~10:15 PM ET): "Yes dffz. Fix as much as you can tonight".
