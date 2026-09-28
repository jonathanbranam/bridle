---
id: xpuc
title: Statusline context as a token count, not just a percent
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The ask

From the human, 2026-09-28:

> file a ticket to add pretty-printed human number to statusline for context
> in addition to percent; 10k, 234k, 1.2M (if we every get a model with >1M
> context).

## Where it lands

- `render_line` in `crates/bridle/src/statusline.rs` prints only
  `ctx {pct}%`. The report already carries `context_used_tokens` and
  `context_max_tokens` (from `context_window`, per the resolved
  [[statusline-real-context-and-a-tighter-layout-s8kn|s8kn]]).
- `bridle agents` already abbreviates its CONTEXT column with
  `format_tokens` in `crates/bridle/src/commands.rs` (`78.9k`, `137k`). It
  has no `M` step, and prints raw digits below 10,000.

## Notes

- `context_used_tokens` is absent after a compact until the next turn ends
  (see `null_current_usage_after_compact_reports_no_used_tokens`), so the
  count needs a fallback when the percent is present but the count isn't.
