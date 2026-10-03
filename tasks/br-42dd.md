+++
id = "br-42dd"
title = "Worktree setup command: run a project's install step in each new worktree"
kind = "feature"
state = "integrated"
created_at = "2026-09-28T23:52:23.825Z"
updated_at = "2026-09-29T01:35:01.547513Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

source: docs/questions/open/onboarding-survey-track-web-and-harness-u8sm.md, section 8 item 4
(track-web stage 1); design sketch in docs/design/worktrees-and-ports.md (`setup = "npm
install --prefer-offline"`). Track-web onboarding is the human's second-priority project;
without this, a fresh worktree of a Node project has no node_modules and a worker can't run
the project's check.

Nothing exists today: no `setup` in crates/bridle-daemon/src/config.rs (grep confirms).

Add a project setting, in the existing worktree-related config section (look at where
`base`/`warm_target`/worktree settings live in config.rs; deny_unknown_fields applies; don't
invent a second section if one exists): `setup = "<shell command>"`, optional, default unset
(no-op, so bridle's own project and meta-notes are unchanged).
- After `git worktree add` and the warm-target copy (crates/bridle-daemon/src/worktree.rs;
  the b7cz change), run the command via `sh -c` with the worktree as cwd, only for worktree
  roles (workers), never for the main clone. Run it off the async runtime (tokio process or
  spawn_blocking); never block the runtime.
- Time limit (default 10 minutes; a `setup_timeout_secs` knob only if trivial). On non-zero
  exit or timeout: fail the spawn with an error naming the command, exit status and the last
  ~20 lines of its output, and clean up exactly as the existing spawn-failure path cleans a
  failed worktree creation (read that path; reuse it, don't invent new cleanup).
- Log the command and duration at info; capture stdout/stderr (bounded) for the error.
- Env for the command: inherits the daemon's, minus the BRIDLE_* agent identity/token
  variables (a setup script must not see an agent token; check how the agent process env is
  built and reuse that filtering if any).
Do NOT add copying of gitignored files (.env, .mcp.json holding a token): whether workers
get those is an open question for the human (u8sm Q8). Do not add paired-layout or port
features from the sketch.

Acceptance: `just check` passes; tests with a temp repo: setup runs in the new worktree cwd
(e.g. writes a marker file), doesn't run when unset, a failing command fails the spawn with
the output in the error and no leftover worktree, a timeout fails the same way (use a tiny
timeout in the test), BRIDLE_TOKEN not visible to it. Docs: worktrees-and-ports.md (mark
`setup` as built, trim the unbuilt layout/pair keys note if it misleads), roles-and-config.md
or wherever worktree config is documented, and the CHANGELOG if the repo keeps one.

Out of scope: paired/linked layouts, port registry, copying secrets, per-role setup,
caching node_modules between worktrees (uu5e's spike).

Size: small-medium. Model: Sonnet.

## Thread

### note · agent:pm-1 · 2026-09-29T01:35:01.547Z
integrated: fadc11f
