+++
id = "br-sf79"
title = "Stamp every prompt with the time it was sent: base UserPromptSubmit hook; 'bridle session' passes layer hooks too"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T23:38:07.774Z"
updated_at = "2026-10-04T00:26:10.683232Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
branch = "bridle/prompt-stamp"
commit = "daf7e62383af71739051bb5ea8b3dc1e979b5182"
summary = "Added workflow/base/hooks/UserPromptSubmit.json (date stamp 'Message sent: ...'); bridle session (session.rs) now resolves Config::layer_hooks for the cwd project and merges it into its --settings via with_layer_hooks (arrays concatenate, bridle's own first; bad files already skipped by layer_hooks). Test: merge keeps bridle's hooks and lean settings. Docs: workflow-layers.md, cli.md, CHANGELOG. Caveat: UserPromptSubmit firing for stream-json input in spawned agents is unverified (documented); no live run done."
+++

Human-approved, asked directly of the orchestrator (2026-10-03, ~7:45 PM ET): "We need to inject the time when I send the message because he's tracking the times very accurately." Origin: the NUC orchestrator (meta-notes m-0288): the human wants a UserPromptSubmit hook for every role: date '+Message sent: %a %Y-%m-%d %H:%M %Z'.
Today: layer hooks (<layer>/hooks/<event>.json; project layer root .bridle/) reach every spawned agent via --settings (c4cd9af, br-01a4, workflow-layers.md "Layer hooks are live at spawn"). But 'bridle session orchestrator|advisor' (crates/bridle/src/session.rs) builds its --settings from bridle's own hooks only (focus gate, focus reply, session note), so the interactive sessions, where the human actually types, never get layer hooks.
Goal: (1) workflow/base/hooks/UserPromptSubmit.json with one command hook: date '+Message sent: %a %Y-%m-%d %H:%M:%S %Z' (stdout of a UserPromptSubmit hook is added to the prompt's context). (2) 'bridle session' resolves the same overlay (sync::discover_hooks_lossy, base, packs, project) and merges it into its --settings beside bridle's own hooks, arrays concatenated, bridle's first, the same rules as spawn (skip a bad file with a warning; drop entries already in committed .claude/settings.json). (3) Check spike 01 / the contract notes for whether UserPromptSubmit fires for stream-json input in spawned agents; if it's unverified, say so in the docs and the ticket rather than assuming (no live run without the orchestrator's go).
Tests: session settings include a layer hook and keep bridle's own; a malformed hook file is skipped. Docs: workflow-layers.md (session sessions get layer hooks), cli.md (bridle session), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none.

## Thread

### note · agent:pm-1 · 2026-10-03T23:38:17.727Z
Planned, tier 2 (after the two k7tm tasks). Touches crates/bridle/src/session.rs, unlike br-8eyu/br-8e5v, so it can run alongside them. Model: Sonnet.

### note · agent:prompt-stamp · 2026-10-04T00:15:38.088Z
done: base UserPromptSubmit time stamp hook; bridle session merges layer hooks into --settings (stream-json firing unverified, documented); just check green (1114 tests); f3663a3

### note · agent:manager-2 · 2026-10-04T00:15:42.782Z
Diff reads fine. main moved (br-8eyu etc.); merge it, run just check, message me sha and result. Make sure 'bridle task show br-sf79' has your summary.

### note · agent:prompt-stamp · 2026-10-04T00:23:11.257Z
main merged, tip 9e0d7d4. just check: 1115 run, 1114 passed, 1 failed: lifecycle_test spawn_child_orphan_is_swept_on_stop (timed out at 64s while the machine was loaded; unrelated to this change). Re-ran it alone: passes in 3s. So the full check did not come back fully green; your call whether to land on that. Summary is on the task.

### note · agent:manager-2 · 2026-10-04T00:23:13.993Z
Rerun the full just check once more (the other worker's tests were loading the machine); if it is fully green, message me the sha and result.

### note · agent:prompt-stamp · 2026-10-04T00:24:37.412Z
done: just check fully green (1115 run, 1115 passed, 5 skipped); tip 9e0d7d4 (main merged)

### note · agent:manager-2 · 2026-10-04T00:24:41.992Z
integrated: daf7e62383af71739051bb5ea8b3dc1e979b5182 (branch bridle/prompt-stamp)

### note · agent:manager-2 · 2026-10-04T00:26:10.683Z
cleanup: removed agent prompt-stamp, branch bridle/prompt-stamp
