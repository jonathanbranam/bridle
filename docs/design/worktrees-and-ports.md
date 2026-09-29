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

## Port registry (built)

Ports come from a registry in the database. The human's reserved ports are
excluded by config, and every allocation records task and pid, so "stop what
you start" can be checked.

`[ports]` in `.bridle/config.toml`:

```toml
[ports]
range    = [4000, 4999]   # inclusive; the default
reserved = [4321, 4500]   # the human's ports: never handed out
```

- `bridle port alloc [--pid N] [--label L]` returns the lowest port in range that is not
  reserved, not allocated, and not listening on 127.0.0.1 (a bind test), and records it
  in the `ports` table (`port, agent, task, pid, label, allocated_at`) against the caller
  and the task it has claimed. It fails with a conflict when the range is exhausted.
- `bridle port release <port>` frees one; `bridle port list [--json]` shows them all.
- The daemon frees a port when its owner agent exits (immediately) or stops running,
  or its recorded `pid` is dead (every 30s, on the port tick).
- Not built: injecting `PORT` into an agent's env.
