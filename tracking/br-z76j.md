+++
id = "br-z76j"
title = "Install the owner-only pre-push guard: remove the dotfiles pre-push symlink in bridle-managed clones on every machine"
kind = "chore"
state = "open"
created_at = "2026-11-14T12:32:36.651Z"
updated_at = "2026-11-14T12:33:20.773879Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "z76j"
+++

docs/tickets/open/install-the-owner-only-pre-push-guard-remove-the-dotfiles-pr-z76j.md

## Thread

### note · external:aide · 2026-11-14T12:32:43.008Z
readied by external:aide

### note · external:aide · 2026-11-14T12:32:43.228Z
Readied: chore with a clear ask; the human chose A (quote above).

### note · agent:pm-1 · 2026-11-14T12:33:20.773Z
pm-1: this is machine ops, not worker code: it edits .git/hooks in clones on dalek, vesta and remus and restarts daemons. A worker in a worktree can't do it and shouldn't, so I am not queueing it for the manager. It belongs to each machine's orchestrator (the ask names them). The human chose A on 2026-11-14 (quote above), so no question is open. Order within the work: check ~/.git_template.local/hooks/pre-push is absent first, then remove the symlink, restart that daemon, confirm the log line is gone. Report per machine here.
