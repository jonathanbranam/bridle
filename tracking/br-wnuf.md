+++
id = "br-wnuf"
title = "Each landing leaves its integrate/<task> scratch branch behind: over 500 piled up"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T02:01:45Z"
updated_at = "2026-10-20T05:15:21.277152Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wwnuf",
]
branch = "bridle/wwnuf"
commit = "dd1409846782ee98703e88dcef17200ea4aa0458"
summary = "land() now detaches the integration worktree and deletes integrate/<task> when a landing ends (landed or failed); integrator::sweep_scratch_branches deletes leftover integrate/* at daemon start (abandoned landings; takes an in-progress task list, empty at start). worktree::sweep_merged_branches, called from the disk tick after the worktree sweep, deletes local bridle/* branches that are merged into the integration branch, checked out nowhere, branch of an integrated/dropped ticket and of no agent. Caveat: git branch --merged misses squash-landed branches that never merged main in, so those stay. Tests with temp repos; docs in roles-and-config.md."
ticket = "wnuf"
+++

docs/tickets/open/each-landing-leaves-its-integrate-task-scratch-branch-behind-wnuf.md

## Thread

### note · external:aide · 2026-10-19T02:01:56.786Z
Readied by aide: a bug with one clear fix (rule ready-without-asking).

### note · agent:pm-1 · 2026-10-19T02:02:32.977Z
Brief (pm-1). Model: Sonnet. Low priority (clutter only, no disk or speed cost); queue it late. Do the ask: in crates/bridle-daemon/src/integrator.rs, after a landing ends (landed, failed or abandoned) switch the integration worktree off integrate/<task> and delete the branch; at daemon start delete integrate/* branches whose landing is over, never the one in progress. Also delete local bridle/* branches that have no agent and no worktree and whose ticket is integrated or dropped and whose commits are on the integration branch (use git branch --merged against the integration branch, not main). Remote branches are br-t7ya's. Tests with temp git repos: each end state deletes; an in-progress landing's branch and an unmerged bridle/* branch stay. Docs: the integrator design doc, CHANGELOG. Acceptance: just check. Overlaps br-mhfx's worktree cleanup: run after it, not alongside.

### note · agent:wwnuf · 2026-10-20T05:15:12.499Z
built: 3cdb2e5a (branch bridle/wwnuf)

### note · agent:wwnuf · 2026-10-20T05:15:12.973Z
done: integrate/* scratch branches deleted after landing and at start; merged agentless bridle/* swept on disk tick; just check exit 0, 1936 tests; 3cdb2e5a

### note · agent:manager-2 · 2026-10-20T05:15:21.277Z
integrated: dd1409846782ee98703e88dcef17200ea4aa0458 (branch bridle/wwnuf)
