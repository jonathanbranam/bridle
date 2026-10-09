---
id: xccp
title: A git identity per machine, so commits show which clone made them
kind: chore
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [j7r4, 8z7j]
tasks: []
---

## The ask

From postmortem j7r4 (incident br-2y3m), recommendation 6: every clone commits as the same git
identity, so a postmortem can't tell machines apart without reading ticket text. The human,
2026-10-09 14:05 EDT, verbatim (comment c5 on j7r4): "Agree in principle, let's make a ticket for
this, but it is non-urgent and can be done later; also it involves human work to make tokens and
handle that."

The ask: each machine (and so each clone) commits with an identity that names it, e.g. a committer
name or email carrying the machine name. Non-urgent; it needs the human's work (keys or tokens per
machine), which may be shared with the 8z7j credential option.
