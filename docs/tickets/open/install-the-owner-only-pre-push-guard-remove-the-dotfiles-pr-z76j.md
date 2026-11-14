---
id: br-z76j
title: "Install the owner-only pre-push guard: remove the dotfiles pre-push symlink in bridle-managed clones on every machine"
kind: chore
created: 2026-11-14
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## Background

bridle's owner-only pre-push guard (rule one-pusher-for-the-integration-branch) is not installed in dalek's bridle, bridle-ui and track-web clones, and likely others on dalek, vesta and remus. `.git/hooks/pre-push` there is a symlink to ~/dotfiles/git_template/hooks/pre-push; bridle refuses to write through a symlink and logs "owner-only push check not installed" at every daemon start. The dotfiles hook only sources ~/.git_template.local/hooks/pre-push, which doesn't exist on dalek, so it does nothing.

Options put to the human (aide, 2026-11-14): A remove the symlinks in bridle-managed clones on every machine; B bridle chains to an existing hook (like br-ged2); C leave it.

The human, 2026-11-14 (to aide), verbatim: "push guard, agree with A"

## The ask

On each machine (dalek, vesta, remus; each machine's orchestrator for its own clones), in every bridle-managed clone whose `.git/hooks/pre-push` is a symlink into dotfiles: check first that the dotfiles hook does nothing there (no ~/.git_template.local/hooks/pre-push), remove the symlink, restart that project's daemon so it installs the guard, and confirm the "owner-only push check not installed" log line is gone. Report per machine on this ticket. Don't touch clones bridle doesn't manage or the dotfiles repo itself.
