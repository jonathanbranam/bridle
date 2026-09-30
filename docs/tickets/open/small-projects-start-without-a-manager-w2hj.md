---
id: w2hj
title: Small projects start without a manager; the orchestrator starts one when there's work
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [one-daemon-for-several-small-projects-3nkk, a-box-manager-for-many-projects-m6qs]
---

## The ask

The human (2026-09-30, via the advisor), verbatim in
[[a-box-manager-for-many-projects-m6qs|m6qs]]: "let's do just option (b) - a per-project config
that dsiables auto-start; and we let the orch start them if there is work to do or if I ask."

Option (b) is 3nkk's option 2. The 3nkk spike found a small project's idle manager is ~60% of its
spend, mostly the turn each resume after a daemon restart starts.

## Today

Config already does most of it (`docs/design/agent-host/roles-and-config.md`, `[roles.manager]`):

```toml
[roles.manager]
autostart         = false   # no manager at daemon start
resume_on_restart = false   # and an existing one isn't resumed after a restart
```

`autostart = false` alone isn't enough: an existing manager still resumes (and takes that costly
turn) on every restart.

## To do

1. **The orchestrator's role** (`workflow/base/roles/orchestrator.md`): for a project with no
   manager running, start one (`bridle spawn`, role `manager`) when the human asks or when there's
   work (a ready task, a message for the manager, a question), and stop it once the project is idle
   again (nothing ready, nothing running). Maybe a machine-level note of which projects are "small".
2. **Check nothing is lost without a manager**: where "task filed" notes, main-moved notes and CI
   failures go when no manager exists (today they go to the manager); they should wake the
   orchestrator instead, or wait for the next manager.
3. **Per project**: set the two keys above in each small project's `.bridle/config.toml`. The human
   approves each (existing-projects rule); meta-notes and track-web first, on their `bridle-adopt`
   branches.
4. Docs: `roles-and-config.md` notes the pairing; `operating-model.md` the no-manager mode.

## The human on it (2026-09-30)

> I think likely having autostart false will be the norm if this system works well. we'll see what
> happens in practice.

So "small projects" may become every project: build it as the general mode, not a special case.
