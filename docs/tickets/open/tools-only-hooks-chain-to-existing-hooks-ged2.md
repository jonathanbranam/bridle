---
id: ged2
title: tools-only-install should chain to an existing git hook, not refuse
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [hw6c]
---

## The ask

`bridle machine tools-only-install` refuses when the clone already has a `pre-commit` or
`pre-push` hook that isn't bridle's (`crates/bridle/src/tools_only.rs`, "not overwriting it.
Move it aside (or chain to it) and re-run"). The human's clones get hooks from a git template
(thoughtbot dotfiles style), so every clone has them. On 2026-09-30, marking the laptop's
meta-notes clone tools-only, the human deleted both hooks by hand to get past it. The human:

> FYI, not a big deal for this repo, but I do use pre-commit hooks in some repos. Not sure how
> to easily resolve that. This is the thoughtbots symoblik link to central pre-commit and
> pre-push hooks, but I don't have anything there

The template hook that was in the way:

```sh
#!/bin/sh

local_hook="$HOME"/.git_template.local/hooks/pre-commit

if [ -f "$local_hook" ]; then
  . "$local_hook"
fi
```

## Direction (not triaged)

A tools-only clone refuses every commit and push anyway, so the existing hook never gets a
chance to matter there. But deleting it loses it if the clone later stops being tools-only.
One simple option: move the existing hook aside (for example `pre-commit.pre-bridle`) and
have the uninstall path (or a re-run with the clone no longer listed) put it back. The
meta-notes laptop clone lost its template hooks this way; `git init` in it re-copies them from
the template if they're wanted back.
Work status: 1 integrated task(s); 1 dropped: br-3efd.
