---
id: sh-7b3f
title: Fix cart rounding
kind: fix
created: 2026-10-01
repos: [acme-shop]
needs: [sh-4a1c]
related: [sh-2c8d]
---

Round once at the end, not per line. Spec: [[docs/design/specs/cart]]. Blocked on sh-4a1c being confirmed.
