# Worktrees and ports

> **Status (checked 2026-10-03):** Built and in use: worktrees at `<workspace>/wt/<agent>` with `[worktrees] setup`/`copy`/`warm_target` · Built, not wired in: `layout = "root"` and `"paired"` (no project sets them yet); the port registry (`bridle port alloc|release|list`; the daemon frees ports on exit, but no role or rule tells agents to allocate) · Planned: injecting `PORT` into an agent's env

`bridle spawn` creates the task's worktree according to a layout the project
declares, including harness's **paired** sibling layout (research 09 §2.2):

```toml
[worktrees]
layout = "root"                                 # "default" (the default) | "root" | "paired"
root   = "/Volumes/Data/work/pi/wt/{task}"
setup  = "npm install --prefer-offline"
```

Built: `layout` and `root`, in the existing `[worktrees]` table alongside `setup`, `copy` and
`warm_target` (one table, not a `[worktree]`/`[worktrees]` pair). `default` puts the worktree
at `<workspace>/wt/<agent>`, unchanged. `root` is an absolute path template with `{task}`
(the agent's claimed task id, else its name; a fresh spawn has none, so its name), `{agent}`
and `{project}` (the clone's directory name); it must contain `{task}` or `{agent}`, and a bad
one (relative, `..`, unknown placeholder, `layout = "root"` without `root`) is refused when
the config loads. A root may leave the workspace, but never the clone: a path inside it is
refused at spawn. `bridle rm` uses the path recorded on the agent, so it finds the worktree
wherever it was put.

Built: `layout = "paired"` (same `root` template, same validation) creates the project's
worktree at `<root>/<project>` and one member per sibling beside it at `<root>/<name>`:

```toml
[worktrees]
layout = "paired"
root   = "/Volumes/Data/work/pi/wt/{task}"
[worktrees.pair.web]
path = "/Volumes/Data/work/pi-web"              # absolute path to the sibling repo
mode = "worktree"                               # "worktree" (default) | "symlink"
```

`worktree` runs `git worktree add` in the sibling on `bridle/<agent>` from that repo's HEAD;
`symlink` links to its checkout, for read-only use. The agent's cwd is the project's
worktree, and its system prompt names each sibling's path. `setup` runs in the project's
worktree and in each `worktree`-mode member. `bridle rm` removes every member (and, with
`--delete-branch`, the sibling branches), refusing on a dirty member by name unless `--force`;
members are found from the current config, beside the recorded worktree. `pair` needs
`layout = "paired"` and the layout needs at least one `pair`.

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
