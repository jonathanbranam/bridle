---
id: br-36a3
title: Rejected role tokens raise an incident, and a 401 spike is covered (follow-up to nbgp ask 2)
kind: fix
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Follow-up to br-nbgp ask 2, which landed short: rejected role tokens are only logged once at start (warn_if_role_tokens_rejected), with no incident raised, and a spike of 401s at runtime is not covered at all.
