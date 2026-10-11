+++
id = "br-ydyk"
title = "ticket new in another project can write the wrong id prefix (br- in bridle-ui)"
kind = "bug"
state = "pending"
created_at = "2026-10-11T13:39:02.299Z"
updated_at = "2026-10-11T13:40:17.606195Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "ydyk"
+++

docs/tickets/open/ticket-new-in-another-project-can-write-the-wrong-id-prefix-ydyk.md

## Thread

### note · external:orchestrator · 2026-10-11T13:40:17.606Z
Cause confirmed (orchestrator): bridle-ui's daemon is still on v0.5.0 (started 2026-10-07), whose status has no task_prefix, so own_prefix falls back to default_task_prefix(project name) = br and ignores [tasks] prefix = "ui" in .bridle/config.toml. Fix: the fallback should read the project's [tasks] prefix from its config (or refuse rather than guess). Restarting bridle-ui's daemon onto 0.6.1 removes the trigger there; the fallback bug stays for any older daemon.
