---
id: br-cp5p
title: GitHub access by fine-grained personal access tokens with the least permissions
kind: feature
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-3827
theme: security
---

## The ask

The human, 2026-10-11 ~1:20 PM ET (to advisor product-manager), verbatim: "we need a theme/epic for security + hardening; future work in that realm is: * agents need limited access to SSH * I think switch to PAT for github to reduce permissions as well * split admin responsibilities that orch has to a separate machine admin role (related: orch + admin become per-machine) * related is identifying agents that make changes" (the second item).

## What's wanted

Bridle's GitHub access (push, gh CLI, CI watch, releases) uses fine-grained personal access tokens
with the least permissions: per repo, per permission (contents, pull requests, actions read), with
an expiry, instead of the user's SSH key or a broad token.

A design pass first: list every GitHub operation bridle and its agents do and the permission
each needs; whether one token per machine or per role (workers may need none: only the owner's
clone pushes, rule one-pusher-for-the-integration-branch); where tokens are stored (the
credentials file, the keychain) and how they rotate; what changes for git remotes (HTTPS with a
credential helper). Then the human reviews.
