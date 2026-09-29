# Worktrees and ports

`bridle spawn` creates the task's worktree according to a layout the project
declares, including harness's **paired** sibling layout (research 09 §2.2):

```toml
[worktree]
layout = "paired"
root   = "/Volumes/Data/work/pi/wt/{task}"
pair   = { harness = "worktree", "track-web" = "symlink|worktree" }
setup  = "npm install --prefer-offline"
```

Only `setup` is built, as `[worktrees] setup` (see
[[agent-host/roles-and-config#Worktree setup command|Worktree setup command]]); the layout,
root and pair keys above are still design.

Ports come from a registry in the database. The human's reserved ports are
excluded by config, and every allocation records task and pid, so "stop what
you start" can be checked.
