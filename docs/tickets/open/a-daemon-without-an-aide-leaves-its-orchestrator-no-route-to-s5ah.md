---
id: s5ah
title: A daemon without an aide leaves its orchestrator no route to the human
kind: bug
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [m7mp, e9yu]
tasks: []
---

## The ask

Reported by orchestrator@nuc (meta-notes) for the human, 2026-10-06. The human: "Send me an issue about that as well, and then you can just send that directly to the orchestrator."

On the NUC, the meta-notes daemon has no `external:aide` principal. `bridle send external:aide ...` fails with `not_found: no such recipient: external:aide`. The orchestrator role reaches the human only through aide, so on the NUC its only route to the human is the live session. The NUC has no bridle gateway yet either (meta-notes mn-tmjz). Tracked on the NUC as meta-notes mn-mfgq.

Expected: every daemon an orchestrator runs on has an aide, or a defined fallback (for example, the human's inbox). Orchestrator priming says what to do when aide is missing.
