+++
id = "br-z76j"
title = "Install the owner-only pre-push guard: remove the dotfiles pre-push symlink in bridle-managed clones on every machine"
kind = "chore"
state = "open"
created_at = "2026-11-14T12:32:36.651Z"
updated_at = "2026-11-14T12:42:41.789527Z"
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

### note · external:orchestrator · 2026-11-14T12:39:04.172Z
vesta (orchestrator@vesta, m-0663): nothing to change. All five bridle-managed clones (vesta-home, meta-notes, meta-notes-ui, notes, dotfiles-local) already have bridle's owner-only hook (regular file, '# bridle-owner-hook', installed Oct 11/15); no core.hooksPath; no ~/.git_template.local/hooks/pre-push; the daemon logs never logged 'owner-only push check not installed'. The vesta bridle clone (no daemon there) has no pre-push hook; left alone. No restarts.

### note · external:orchestrator · 2026-11-14T12:42:41.789Z
dalek (orchestrator): dotfiles pre-push symlinks removed in bridle, bridle-ui, data-contracts, track-web clones (hooks dirs are real dirs, no core.hooksPath; vendored OpenSpec/beads/beads_rust/swarm-forge left alone; dalek-home already had the guard). Restarted bridle-ui, data-contracts, track-web at 12:41 UTC: each clone now has a regular .git/hooks/pre-push with '# bridle-owner-hook' (exec bridle machine push-check), and no 'owner-only push check not installed' in their daemon logs since. bridle: its daemon has an upgrade restart to 8352cc3 draining on worker nxkj; I'll confirm its hook after that lands. remus: no reply yet.
