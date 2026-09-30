---
id: s8kn
title: "bridle statusline: real context, a tighter layout, and snapshots that arrive"
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
closed: 2026-09-30T05:12:44Z
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

## A likely cause for item 5, from a second conversation (verbatim)

The human ran `echo '{}' | bridle statusline` from a plain shell on
2026-09-28, and a row (id 1, all fields empty) reached `interactive_usage`.
The agent's diagnosis:

> What the test showed: the database dates from 17:41 today, not from when
> you first set up the status line. So "empty table" meant "nothing recorded
> today", not "never". Even so, your interactive sessions have been running
> all day without adding a row, while your shell run added one immediately.
>
> Likely cause: crates/bridle-api/src/discovery.rs:287. When $CLAUDECODE is
> set, the CLI deliberately refuses to use your human token file unless
> $BRIDLE_TOKEN is set. That's a safety rule so an agent running inside
> Claude Code can't pick up your credentials. Claude Code sets CLAUDECODE for
> processes it starts, so bridle statusline probably fails at that step when
> Claude Code runs it. commands.rs:96 then silently discards the error, so
> the line still prints but nothing gets recorded. Your shell doesn't have
> CLAUDECODE set, which is why your manual run succeeded.
>
> I haven't confirmed that Claude Code sets CLAUDECODE for the status line
> command specifically, as opposed to Bash tool commands. Checking that
> should be the first step of the ticket.
>
> Fix options for the ticket:
> - Give the status line a narrow token that can only POST /v1/statusline,
>   and pass it through the command, e.g. BRIDLE_TOKEN=… bridle statusline.
>   This keeps the safety rule intact. Headless workers never run a status
>   line, so the token wouldn't reach them.
> - Or allow the human token for statusline only. That's simpler, but it
>   weakens the rule.
>
> Either way, log the failure instead of swallowing it.
>
> Your test also left one row with all fields empty in interactive_usage.
> It's harmless, but it will appear in bridle usage today.

## Decision: drop recording, don't chase the token gap

The human, via the orchestrator, mid-task on the fix above:

> Drop the recording half of `bridle statusline` entirely. No
> `POST /v1/statusline` call from the statusline command, no statusline
> token, no token file, no `bridle token create statusline` setup. The
> context governor already gets account-wide windows from `get_usage`, and
> the original reason for statusline recording has passed. Also drop the
> working/waiting-counts stretch entirely — reading those needs the same
> under-`$CLAUDECODE` token problem, so skip it, don't build it.

This supersedes the token-file plan item 5 originally called for (a
dedicated `external:statusline` token in a per-workspace file, read
regardless of `$CLAUDECODE`). `bridle statusline` is now purely local: it
parses Claude Code's stdin JSON and prints a line, with no daemon call, no
token, and nothing that can fail beyond unparseable stdin.

`POST /v1/statusline` and the `interactive_usage` table stay in the daemon
as-is — not removed, just unused for now, in case a future need for
per-invocation interactive snapshots resurfaces. If it does, item 5's
token-file plan (and its known gap — an `external:statusline` token isn't
endpoint-scoped, tracked separately in 4eep/a7h3) is still the right shape
for it; it just isn't needed today.

## Why it matters

The human reads the status line all day; today its context figure is wrong
(item 2) and its snapshots, which feed the budget governor's view of
interactive sessions, never arrive (item 5).

## Resolution

All findings have been verified against `crates/bridle/src/statusline.rs`:

1. **Context parsing** (item 1): Confirmed. `context_window()` (lines 81–103) correctly parses `used_percentage`, `context_window_size`, and `current_usage` with its three input fields summed for token accounting.

2. **Exceeds 200k fallback** (item 2): Confirmed removed. `exceeds_200k_tokens` does not appear in the parsing logic. Test `extended_context_model_is_not_capped_at_200k()` (lines 296–316) verifies that 1M-context models at 200k tokens read as 20%, not 100%.

3. **Rate limits** (item 3): Confirmed. `WINDOWS` const (lines 22–27) explicitly lists `five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet`. Parser correctly skips missing or unknown windows.

4. **Cost de-emphasization** (item 4): Confirmed. `render_line()` (lines 181–183) places cost last, parenthesized. Test (line 278) verifies the line ends with ')'.

5. **Recording drop** (item 5): Confirmed. Parsing is pure, no HTTP calls or token handling. `render_counts()` (lines 191–204) uses the read-only token from r7cs (merged at df32928) for counts display. The decision to drop recording has been implemented: no `POST /v1/statusline` from statusline; the daemon's endpoint and `interactive_usage` table remain unused.

Test fixtures use the documented shape (lines 223–279), validating the field names and structure. All changes described in this ticket have landed in production code.

See also the spike findings: `docs/spikes/04-statusline-stdin-schema-findings.md`.
