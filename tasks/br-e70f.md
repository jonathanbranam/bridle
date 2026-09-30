+++
id = "br-e70f"
title = "State push defaults to on for every project (we2r follow-up)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T01:11:54.756Z"
updated_at = "2026-09-30T01:11:57.809110Z"
size = "S"
+++

The human approved pushing bridle/state for every project, including meta-notes and track-web (advisor m-2079; ticket docs/questions/open/push-the-state-branch-we2r.md). br-93ad landed with [state] push = false by default; br-011b (handovers on the state branch, rebuild from origin) lands before you start. Do: 1) make [state] push default to true (crates/bridle-daemon/src/config.rs and wherever the default is documented), keeping push = false as an explicit opt-out; 2) remove the now-redundant explicit push line from bridle's own .bridle/config.toml, and any scaffold in bridle init (crates/bridle/src/init.rs) that writes it, so new projects inherit the default; 3) update docs/design/storage.md, the [state] config docs (roles-and-config.md), doc text in bridle doctor if it mentions the switch, workflow/base/rules/existing-projects.md if it says pushing is off or unapproved (state what the human approved and when: 2026-09-29), the we2r ticket, and CHANGELOG; 4) CONFIRM the first daemon push handles a remote that already has bridle/state from a hand push: local is ahead of or equal to origin, so it is a plain fast-forward (or a no-op when equal): add a test with a bare origin that already has the branch at an ancestor commit, and one where origin equals local (no error, no force, status shows fine); also a project with no remote configured must stay quiet (status says no remote, no WARN spam). Never use force anywhere. Acceptance: just check passes; the tests in 4. Model: Haiku. Out of scope: any other state-push behaviour.
