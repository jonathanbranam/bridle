+++
id = "br-cc25"
title = "P5: paired worktree layout (sibling repos created together)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T08:10:45.474Z"
updated_at = "2026-09-29T08:10:47.848715Z"
+++

Goal (docs/design/worktrees-and-ports.md, harness+track-web): with `[worktree] layout = "paired"` a spawn creates the task's worktree for the project repo AND for each configured sibling repo, side by side under the root: `[worktree.pair.<name>] path = "/abs/path/to/sibling/repo", mode = "worktree"|"symlink"` (worktree: git worktree add on branch bridle/<name> from that repo's own base; symlink: a symlink to the sibling's checkout, read-only use). Layout: <root>/<project> and <root>/<name>; the agent's cwd is the project's worktree; the sibling paths go in the agent's system prompt facts (render_system_prompt in the daemon). Setup command runs in each worktree-mode member. rm/cleanup removes all members (refusing on dirty like today, naming which member). Files: crates/bridle-daemon/src/worktree.rs, config.rs, supervisor.rs spawn path; docs worktrees-and-ports.md, roles-and-config.md.

Acceptance: just check passes; tests with two temp git repos: paired spawn makes both, symlink mode links, rm cleans both, dirty sibling refuses. Model: Sonnet. Runs after the worktree layout config task (same files).
