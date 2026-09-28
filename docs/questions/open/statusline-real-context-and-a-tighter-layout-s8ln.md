---
id: s8ln
title: "bridle statusline: real context, a tighter layout, and snapshots that arrive"
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

`bridle statusline` (Claude Code's `statusLine` command, `cd4ffef`) was built
against guessed field names. **Needs** the spike
[[confirm-statusline-stdin-json-schema-m9wt|confirm the statusline stdin JSON schema]]
(in `docs/spikes/open/`), which the findings below largely answer. What should
the status line show, and how should it parse and report?

The human's words, 2026-09-28:

> I have a new ticket to file on the status line. I don't even remember what
> was there before, but I want to see the actual context for the interactive
> agent. And let's tighten it up: use "5h" and "7d"; keep the model name; add
> the current folder and git branch. My work statusline is pretty good it uses
> a 📁 icon for the folder and some sort of leafy fern branch emoji for the
> branch.
>
> For bridle sessions, IDK if we can figure out which session we are working
> on based on the folder, but if we can, then we should have some indication
> of the work being done (counts) and waiting messages for me (counts).

## Findings, from an agent's review the human passed on (verbatim)

> bridle statusline: context % never shows, and snapshots are never recorded
>
> 1. Context parsing uses field names Claude Code doesn't send.
> crates/bridle/src/statusline.rs, context_window(), reads
> context_window.used_tokens / .max_tokens. The real fields are:
>
> | Field | Meaning |
> |---|---|
> | `context_window.used_percentage` | 0–100, precomputed. Counts input tokens only: input + cache creation + cache read. |
> | `context_window.context_window_size` | Model's context size: 200000, or 1000000 for extended-context models. |
> | `context_window.current_usage.{input_tokens, cache_creation_input_tokens, cache_read_input_tokens, output_tokens}` | Tokens from the last API call. null before the first call and right after /compact. |
> | `context_window.total_input_tokens / total_output_tokens` | Also from the most recent response. These are the current context, not session totals. |
>
> The docs say context_window "describes the live context window from the
> most recent API response."
>
> Suggested fix: show used_percentage directly. Record used tokens (the three
> input fields of current_usage added together) and context_window_size as
> the maximum.
>
> 2. The fallback is actively misleading. Top-level exceeds_200k_tokens: true
> is currently reported as ctx 100%. On a model with 1M context, the display
> jumps to 100% once you pass 200k while 80% of the window is still free.
> This is probably the ctx you saw earlier. Once the real fields are parsed,
> remove this fallback.
>
> 3. Rate limits parse fine, with small gaps. resets_at is epoch seconds, and
> the parser already accepts that. The documented windows are five_hour,
> seven_day and spend_limit. seven_day_opus / seven_day_sonnet aren't in the
> docs. spend_limit is ignored, which is fine for a Max plan. Windows can be
> missing before the first API response, and Claude Code drops each one after
> its resets_at.
>
> 4. The dollar figure: the docs say cost.total_cost_usd is an "estimated
> session cost in USD, computed client-side at list price... May differ from
> your actual bill. Resets to $0 when /clear starts a new session." Consider
> showing it less prominently, or not showing it.
>
> 5. Snapshots never reach the daemon. The workspace database's
> interactive_usage table has 0 rows, even though the status line is in daily
> use. The daemon handler (server.rs:731) looks fine. The client throws away
> every failure: finding the daemon, resolving the token, the HTTP error and
> the 2s timeout (commands.rs:96–98). So the cause isn't known yet. It could
> be discovery from the session's cwd, token resolution, or an auth
> rejection. At minimum the client should log the failure with a tracing
> debug line so it can be diagnosed. This also means the budget governor
> never gets your interactive sessions' readings.
>
> Test fixtures in statusline.rs use the invented shape. Replace them with the
> documented example JSON from the page above.

## Why it matters

The human reads the status line all day; today its context figure is wrong
(item 2) and its snapshots, which feed the budget governor's view of
interactive sessions, never arrive (item 5).
