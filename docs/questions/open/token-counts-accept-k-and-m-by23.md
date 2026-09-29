---
id: by23
title: Token counts in config accept k and M (150k, 1.5M)
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [the-orchestrator-stays-running-fx7x]
---

## The ask

The human, verbatim (2026-09-29, via the advisor), about the `[orchestrator]` config:

> also file a small (xs?) ticket to support parsing k and M for tokens anywhere they are
> specified:
>
> ```
> note_tokens     = 150000
> plan_tokens     = 180000
> handover_tokens = 200000
> ```
>
> I want to write 150k not 150000

## Shape

- Every token count in `.bridle/config.toml` accepts an integer (as now) or a string with `k` or
  `M`: `150000`, `"150k"`, `"1.5M"` (case-insensitive; `k` = 1,000, `M` = 1,000,000, matching the
  `152k` display in dc22de3). Anything else is a config error naming the key.
- Today that's `note_tokens`, `plan_tokens` and `handover_tokens` (`crates/bridle-daemon/src/config.rs`,
  `RawOrchestrator`). One parser beside `parse_duration`, used by any future token key.
- Update the examples in `docs/design/agent-host/orchestrator-supervision.md` and
  `roles-and-config.md` to `150k` style.
