---
id: 8z7j
title: "Only the owner's clone can push the integration branch: enforced, not a rule"
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [j7r4, kt25, xrkh, k6jd, 8ay6, xccp]
tasks: []
---

## The ask

From postmortem j7r4 (incident br-2y3m), recommendation 1: only one clone pushes a project's
integration branch. The human, 2026-10-09 14:03 EDT, verbatim (comment c1 on j7r4):

> Agree. This is correct. We have "project takeover" to transfer a project between machines /
> clones. We just need to enforce this mechanically - either do a read-only clone or do something
> with git or PAT tokens or somehow enforce this. We can write a rule, but it should be impossible
> for a different clone under bridle to push.

The ask: make it **impossible**, not just a rule, for any clone under bridle other than the
project's owner (the machine in `owner.toml`, changed by `serve --take-over`) to push the
integration branch to origin. Taking a project over moves the right to push with it.

Needs a design (the designer role) before it is built; the human picks. Options the human named
or that fit:

- A pre-push hook bridle installs in every clone it manages, refusing a push to
  `origin/<integration>` unless this machine is the owner (agents already can't pass
  `--no-verify` if their deny lists say so; the human can).
- Non-owner clones get a push URL that can't push (`git remote set-url --push origin no-push`),
  and take-over switches it back.
- Credentials: only the owner machine holds a key or token with write access (a deploy key or
  fine-grained PAT per machine), moved at take-over. Strongest; needs human work per machine
  (see xccp).

Also the rule ("one pusher for the integration branch", `workflow/base/rules/`) as the written
form, and operating-model.md's "Merging" step 4 says who pushes. Part of the machine setup
workstream: the Windows PC is a third clone.
