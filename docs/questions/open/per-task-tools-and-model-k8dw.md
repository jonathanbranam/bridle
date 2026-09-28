---
id: k8dw
title: Let a task say which tools and model its worker needs
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [a-new-name-for-the-project-geem]
---

## What happened

The geem research task went to an ordinary worker, which has no WebSearch or
WebFetch (worker `allowed_tools` in `.bridle/config.toml`; headless `claude`
denies anything not listed). It wrote the catalogue from memory and merged it
marked unverified.

The human, verbatim (2026-09-28):

> I'm OK with workers not having web; I wasn't sure if they did or not; file a new work
> item that some tickets may require different tools than the default (as well as a
> specific model) and we should allow that when writing the ticket.

## Why it matters

A task that needs a tool its role lacks fails quietly: the worker carries on
without it. Today the only ways around that are a new role (config, read only at
daemon startup) or `bridle spawn --model`, which covers the model but not tools.

## Notes

- `bridle spawn` has `--model` but no tools option; tools come only from the
  role's `allowed_tools`/`disallowed_tools`
  (docs/design/agent-host/roles-and-config.md).
- The ask: whoever writes the ticket or task can state the tools (beyond the
  role's defaults) and the model it needs, and the worker that takes it gets them.
- Replaces the `researcher` role the orchestrator proposed to the product
  manager (m-0778) for the same gap.
- The human, verbatim (2026-09-28, on k8dw and 2ty9): "KISS for both of these.
  Also on general I trust Claude agents so I don't think we need to go overboard
  in restricting their access too much."
