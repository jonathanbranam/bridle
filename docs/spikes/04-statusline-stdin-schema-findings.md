# Spike 04 findings — statusline stdin JSON schema

Date: 2026-09-28   Rust: 1.98.1 (edition 2024)

## Verdict

**Confirmed.** The schema has been reverse-engineered and verified against Claude Code's own documentation, and the implementation at `crates/bridle/src/statusline.rs` parses it correctly. The field names were guessed at first (spike reason), but are now confirmed against the documented schema.

## Answers

| Field | Type | Notes |
|---|---|---|
| `model.display_name` / `model.id` | string | Display name preferred; falls back to model ID |
| `session_id` | string | Unique identifier for the Claude Code session |
| `cost.total_cost_usd` | number | Estimated session cost, list-price basis; resets on /clear |
| `context_window.used_percentage` | number (0–100) | Precomputed; input tokens only (input + cache creation + cache read) |
| `context_window.context_window_size` | number | Model's context size (200,000 or 1,000,000 for extended-context models) |
| `context_window.current_usage.input_tokens` | number | Input tokens from the most recent API call; null before first call and after /compact |
| `context_window.current_usage.cache_creation_input_tokens` | number | Cache creation tokens from the most recent API call; null before first call and after /compact |
| `context_window.current_usage.cache_read_input_tokens` | number | Cache read tokens from the most recent API call; null before first call and after /compact |
| `context_window.current_usage.output_tokens` | number | Output tokens from the most recent API call; null before first call and after /compact |
| `context_window.total_input_tokens` | number | Cumulative input tokens for the session (from the most recent response) |
| `context_window.total_output_tokens` | number | Cumulative output tokens for the session (from the most recent response) |
| `rate_limits.five_hour` | object with `used_percentage` and `resets_at` | Rate limit for the 5-hour window |
| `rate_limits.seven_day` | object with `used_percentage` and `resets_at` | Rate limit for the 7-day window |
| `rate_limits.seven_day_opus` | object with `used_percentage` and `resets_at` | Rate limit for the 7-day Opus window (not in docs) |
| `rate_limits.seven_day_sonnet` | object with `used_percentage` and `resets_at` | Rate limit for the 7-day Sonnet window (not in docs) |
| `rate_limits.*.used_percentage` | number (0–100) | Utilization percentage for the window |
| `rate_limits.*.resets_at` | number (epoch seconds) or string (RFC3339) | When the window resets |
| `workspace.current_dir` | string (path) | Session's working directory; falls back to top-level `cwd` if absent |
| `cwd` | string (path) | Top-level working directory, used if `workspace.current_dir` is absent |

## Evidence

Implementation: `crates/bridle/src/statusline.rs` (lines 72–104 for `context_window()`, lines 44–59 for rate-limit windows).

Test fixtures use the documented shape: `parses_the_documented_shape()` (lines 223–279) validates `used_percentage`, `context_window_size`, `current_usage` with its three input fields, `resets_at` as both RFC3339 string and epoch seconds.

Notable implementation details:
- `WINDOWS` const (lines 22–27) explicitly lists the rate-limit windows known to bridle: `five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet`.
- `current_usage` is summed (input + cache creation + cache read) for token accounting; it is `null` before the first API call and after `/compact`.
- `exceeds_200k_tokens` fallback (mentioned in original findings) is no longer used; extended-context models now report accurately via `used_percentage` and `context_window_size`.
- Parser is tolerant: missing fields and unknown windows are skipped, never an error.
