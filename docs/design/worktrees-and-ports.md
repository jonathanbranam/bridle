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

Ports come from a registry in the database. The human's reserved ports are
excluded by config, and every allocation records task and pid, so "stop what
you start" can be checked.
