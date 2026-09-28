---
id: vxp6
title: Spike: how Claude Code loads path-scoped rules
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/design.md` §4.4 @ c192bfc, the rendering table:

> | path-scoped rules | L4 component rules, rendered as nested/path-scoped rule files so they load only when the agent works in that path (**verify** the exact Claude Code mechanism) | no |

## Why it matters

Component rules ([[docs/design/workflow-layers|workflow layers]]) and the
standing rule on `design/explore/**` ([[docs/design/explorations|explorations]])
both rely on it.

## Notes
