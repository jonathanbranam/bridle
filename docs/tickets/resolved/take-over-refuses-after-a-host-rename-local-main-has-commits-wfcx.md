---
id: br-wfcx
title: "take-over refuses after a host rename: local main has commits the pre-push hook itself held back"
kind: bug
created: 2026-10-14
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
theme: multi-machine
resolved: 2026-10-14T03:53:38Z
---

## The ask

Submitted by the meta-notes aide on vesta. The human, verbatim: "yes file the takeover refusing on main as a bug".

## What happened (2026-10-13, bridle 0.7.1)

1. The human renamed the machine nuc -> vesta in ~/.bridle/config.toml at 8:59 PM ET. owner.toml on bridle/state still said host = nuc for all four projects on the machine, so the one-pusher pre-push hook refused every push from vesta ("owned by nuc ... bridle project take-over").
2. Agents kept committing while pushes were refused. meta-notes had 1 commit on local main (fe6e60f, the incident for this very outage), and notes had 2 (the human's Tuesday daily note).
3. With the daemons stopped, `bridle projects take-over --force` in each clone gave:
   - meta-notes: `take-over refused: local main (fe6e60f...) is not an ancestor of origin's (90fc1df...); sort it out by hand, bridle never rebases or forces`
   - notes: the same (local 7fe88cd vs origin 7e85d5e)
   - dotfiles-local (local main already pushed): succeeded.

That's a deadlock. Take-over needs local main already on origin, but the push is refused until take-over has run. The message is also wrong: local main is AHEAD of origin (origin's main is its ancestor), not diverged, so there is nothing to "sort out".

Workaround the human used, per clone: `git branch hold main; git reset --hard origin/main; bridle projects take-over --force; git merge --ff-only hold; git push origin main`.

## The ask

- When local main is a fast-forward of origin's main, take-over should accept it, then push it (or say to push) once ownership has moved. Refuse only real divergence.
- The refusal message should say which case it is (ahead vs diverged) and give the steps.
- Related: a host rename in config.toml alone breaks pushes on every project at once. Consider detecting it, or document "rename = take-over every project, push first".

## A second, smaller issue in the same run

take-over's "move [projects] on every machine" step ran `ssh -o BatchMode=yes remus ...`, and remus had Tailscale SSH in "check" mode, which waits for a browser login. BatchMode doesn't stop that wait, and there's no ConnectTimeout or overall timeout, so the take-over hung with no output until the human interrupted it. Add a timeout, and name the machine it's waiting on.

