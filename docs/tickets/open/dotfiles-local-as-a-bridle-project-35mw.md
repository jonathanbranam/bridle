---
id: 35mw
title: dotfiles-local as a bridle project, directly on main
opened: 2026-09-30
repos: [bridle, dotfiles-local]
changes: []
specs: []
needs: []
see: [nuc-recovery-on-boot-4r3k, one-daemon-for-several-small-projects-3nkk]
---

## The ask

The human, verbatim (2026-09-30, via the advisor):

> file a ticket to make my dotfiles-local a bridle project, go directly on main. I have other work
> there to do as well [...] the first task for dotfiles-local is these fixes, the next task is
> per-arch configuration. I'm deferring per-box configuration for now, I think per-arch is OK, but
> I am considering per-box as well

**Directly on main** is the human's explicit approval to skip the `bridle-adopt` trial branch
(63rv) for this repo: trunk on `main` (rxe8's first pattern).

## The repo

`git@github.com:jonathanbranam/dotfiles-local`, cloned at `~/dotfiles-local` on the laptop (clean,
`main`) and on the NUC (`main`, 3 uncommitted changes, an `https` remote: check it can push).
`~/.tmux.conf.local` links to it, and the tmux plugin list lives there. `~/.tmux.conf` links to
`~/dotfiles/tmux.conf`, a separate repo.

A `dotfiles-local` daemon was already started on the NUC once (00:43 UTC today): its stale
`daemon.json` sits at `~/.bridle/daemon.json`, i.e. with the workspace set to the home directory.
Clean that up and pick a proper workspace.

## Tasks, in order

1. **The tmux fixes** (the "full recovery on boot" plan, 4r3k part 1):
   - the tmux-resurrect fix: today an uncommitted edit only on the laptop (and at work) in
     `~/.tmux/plugins/tmux-resurrect` (`save.sh` writes `:#{pane_title}`, `restore.sh` strips the
     `:`, so an empty pane title doesn't shift the fields). Put it in a fork and point tpm at it (or
     vendor it), so every box gets it from the repo;
   - add tmux-continuum: restore on start, save every 15 minutes, start tmux at boot (systemd on
     Linux);
   - save and restore the `@bridle` pane tags with resurrect's hooks (4r3k).
2. **Per-arch configuration** (e.g. macOS arm64/x86_64 vs Linux). Per-box configuration is
   deferred; the human is still weighing it.

## Human-only

- `sudo loginctl enable-linger jbranam` on the NUC (`Linger=no` today), so the systemd user manager
  and tmux start at boot without a login.
