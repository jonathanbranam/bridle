---
id: br-eas4
title: "bridle serve ignores --project for the repo: say so when --project and the folder disagree"
kind: bug
created: 2026-10-12
created_by: external:advisor/nuc-rename
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-r2nr]
resolved: 2026-10-16T10:02:58Z
---

## The ask

The human, 2026-10-11 ~10:50 PM ET (to advisor nuc-rename), verbatim: "please file a ticket to
improve that error message. It should indicate that the --project and folder don't agree, I think;"

What happened (the NUC rename, r2nr step 4), run from the meta-notes clone:

```
jbranam@nuc:meta-notes main % bridle --project meta-notes-ui serve --take-over
2026-10-12T02:47:32.201222Z  WARN bridle::serve: migration 0006-backfill-ticket-kind skipped, will retry at next start: main is a protected branch of meta-notes; check out the project's trial branch (e.g. bridle-adopt) and rerun
error: a daemon is already running for this workspace (pid 237878)
```

The human meant to serve meta-notes-ui; the error names neither project, so it reads as if
meta-notes-ui were already running.

## Findings (advisor nuc-rename, 2026-10-11)

- `bridle serve` takes the repo from `--repo` or the cwd (docs/design/agent-host/daemon.md,
  "takes the repo from `--repo` or the cwd"), but passes `--project` / `$BRIDLE_PROJECT` on as the
  project name (crates/bridle/src/serve.rs:41, `project: cli.project.clone()`). So it tried to serve
  the meta-notes repo under the name meta-notes-ui.
- It was refused only because meta-notes's own daemon was up
  (`refuse_if_already_running`, crates/bridle-daemon/src/lib.rs:1290-1311). The name check there
  passed because meta-notes-ui was stopped. Had meta-notes been stopped too, it looks like it would
  have started meta-notes's repo registered as meta-notes-ui (not tried).
- The migration warning came from the cwd's project too (meta-notes), which adds to the confusion.

## Possible fix (for whoever plans it)

When `--project` (or `$BRIDLE_PROJECT`) names a project other than the one the repo from
`--repo`/cwd belongs to, `serve` refuses before anything else (before migrations), saying both:
e.g. "--project meta-notes-ui, but this folder is project meta-notes (<path>); cd to
meta-notes-ui's clone (<registry path, if known>) or drop --project". Alternative: resolve the repo
from `--project` via the registry. Either way, the "already running" error should name the project
and workspace it found.
