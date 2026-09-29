+++
id = "br-cc25"
title = "P5: paired worktree layout (sibling repos created together)"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T08:10:45.474Z"
updated_at = "2026-09-29T09:03:20.752021Z"
branch = "bridle/paired-worktree"
commit = "514a1b96357e24a49bcf2b1061576bfdc717983b"
summary = 'Added [worktrees] layout = "paired" with [worktrees.pair.<name>] path/mode (worktree|symlink): spawn creates <root>/<project> plus each sibling (worktree on bridle/<agent> from sibling HEAD, or symlink), setup runs in each worktree member, sibling paths are in the system prompt, rm removes all members and refuses on a dirty one by name. Members are found at rm time from current config beside the recorded worktree (no schema change). Config key is [worktrees] (existing table), not [worktree] as the brief wrote. Docs and CHANGELOG updated.'
+++

Goal (docs/design/worktrees-and-ports.md, harness+track-web): with `[worktree] layout = "paired"` a spawn creates the task's worktree for the project repo AND for each configured sibling repo, side by side under the root: `[worktree.pair.<name>] path = "/abs/path/to/sibling/repo", mode = "worktree"|"symlink"` (worktree: git worktree add on branch bridle/<name> from that repo's own base; symlink: a symlink to the sibling's checkout, read-only use). Layout: <root>/<project> and <root>/<name>; the agent's cwd is the project's worktree; the sibling paths go in the agent's system prompt facts (render_system_prompt in the daemon). Setup command runs in each worktree-mode member. rm/cleanup removes all members (refusing on dirty like today, naming which member). Files: crates/bridle-daemon/src/worktree.rs, config.rs, supervisor.rs spawn path; docs worktrees-and-ports.md, roles-and-config.md.

Acceptance: just check passes; tests with two temp git repos: paired spawn makes both, symlink mode links, rm cleans both, dirty sibling refuses. Model: Sonnet. Runs after the worktree layout config task (same files).

## Thread

### note · agent:manager-2 · 2026-09-29T09:03:20.752Z
integrated: 514a1b96357e24a49bcf2b1061576bfdc717983b (branch bridle/paired-worktree)
