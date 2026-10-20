+++
id = "br-wnuf"
title = "Each landing leaves its integrate/<task> scratch branch behind: over 500 piled up"
kind = "bug"
state = "claimed"
created_at = "2026-10-19T02:01:45Z"
updated_at = "2026-10-20T04:56:54.449410Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wwnuf",
]
ticket = "wnuf"
+++

docs/tickets/open/each-landing-leaves-its-integrate-task-scratch-branch-behind-wnuf.md

## Thread

### note · external:aide · 2026-10-19T02:01:56.786Z
Readied by aide: a bug with one clear fix (rule ready-without-asking).

### note · agent:pm-1 · 2026-10-19T02:02:32.977Z
Brief (pm-1). Model: Sonnet. Low priority (clutter only, no disk or speed cost); queue it late. Do the ask: in crates/bridle-daemon/src/integrator.rs, after a landing ends (landed, failed or abandoned) switch the integration worktree off integrate/<task> and delete the branch; at daemon start delete integrate/* branches whose landing is over, never the one in progress. Also delete local bridle/* branches that have no agent and no worktree and whose ticket is integrated or dropped and whose commits are on the integration branch (use git branch --merged against the integration branch, not main). Remote branches are br-t7ya's. Tests with temp git repos: each end state deletes; an in-progress landing's branch and an unmerged bridle/* branch stay. Docs: the integrator design doc, CHANGELOG. Acceptance: just check. Overlaps br-mhfx's worktree cleanup: run after it, not alongside.
