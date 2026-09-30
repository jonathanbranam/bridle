---
id: 2ty9
title: Give an authorised worker a secret and network access other agents don't get
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [9ee5f5d, f925ef2, 5d39325]
specs: []
needs: []
see: [per-task-tools-and-model-k8dw]
closed: 2026-09-30T05:12:44Z
---

## What happened

The human, verbatim (2026-09-28, after filing k8dw):

> Another note and vote for worker permissions - for my game development will have an
> agent generate images and animations using an external API from pixel lab that I pay
> for. That agent must be able to access that domain and make API calls and must have
> the token but other agents don't need that and probably shouldn't have the token.
>
> So bridle may need a little more in the permissions department. An authorized worker
> should be able to either request a token or should Be provided one on startup. Other
> work I have uses an external api for other models that I also pay for through DeepInfra

## Why it matters

Paid API keys (PixelLab, DeepInfra) should reach only the worker whose task needs
them. Today an agent's environment and permissions come from its role, so a key
given to one worker is given to every worker of that role, and a key in the
daemon's environment may reach all of them.

## Notes

- Two parts: the secret itself (provided at startup, or requested by an
  authorised worker), and permission to reach the API's domain (a tool or
  network allowance, the same per-task question as k8dw).
- Known uses: PixelLab (images and animations for game development) and
  DeepInfra (other models).
- The human, verbatim (2026-09-28, on k8dw and 2ty9): "KISS for both of these.
  Also on general I trust Claude agents so I don't think we need to go overboard
  in restricting their access too much."

## Resolution

Resolved by f925ef2 (9ee5f5d): `bridle spawn --env KEY=VALUE` gives one spawn a secret no other agent gets, persisted on that agent and reapplied on renew/resume (5d39325). Network access needs nothing new (every role has `Bash`; `--allow-tool` can scope `WebFetch` to a domain). The answer lives in docs/design/agent-host/roles-and-config.md ("Per-spawn secrets") and docs/design/cli.md.
