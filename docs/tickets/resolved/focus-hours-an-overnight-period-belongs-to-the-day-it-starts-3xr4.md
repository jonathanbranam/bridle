---
id: 3xr4
title: "Focus hours: an overnight period belongs to the day it starts, so one block covers the night"
kind: bug
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [focus-hours-quiet-and-locked-cvaq, per-project-focus-hours-a-project-s-own-schedule-or-opt-out-mfgb]
tasks: [br-14d6]
closed: 2026-10-09T23:11:08Z
---

## The ask


The human, verbatim (2026-10-01, via the advisor):

> file a bug to change the way that the focus hours time works. The way it works right now is
> super confusing. The, the agents are telling me quiet time ends at midnight because I had to set
> up one that started in the evening and ran until midnight and I had to create a different focus
> time block that starts at midnight and goes until the morning and it's just massively confusing
> so we need to change how that works and then I need to update the focus time definition I guess
> the definition as it is now will still work but the rule is if the two times are both if start
> is before end, then it's during the same day. If end is before start, then it stretches over a
> day and into the next day. And I think that's probably the easiest way to solve it. It'll make
> setting that up a lot more straightforward for me too.

## Today (advisor, checked 2026-10-01)

- `in_window` (`crates/bridle-daemon/src/config.rs:643`, shared by `[[focus]]` and
  `[[budget.schedule]]`) already wraps when `end < start`, but it checks `days` against **now's**
  weekday. So after midnight it matches the *next* day's entry.
  [[docs/design/agent-host/roles-and-config|roles and config]] (line 179) says so, and its
  example says "a range crossing midnight needs two blocks".
- So the human's `~/.bridle/config.toml` splits each night in two: `weekday-sleep` (sun–thu,
  21:30–00:00), `weekend-sleep` (fri–sat, 23:30–00:00), and `everyday-sleep` (all, 00:00–06:00).
- The gate reports the matched block's own end (`crates/bridle/src/focus.rs`, "QUIET HOURS
  ({name}) until {end} ET"), so in the evening block agents say quiet hours end at 12:00 AM, though
  the next block starts right then.

## The fix

1. **A period belongs to the day it starts.** `start < end`: the same day. `end < start`: from
   `start` on a listed day to `end` the next day, whatever that next day is. The part after
   midnight is matched against the **previous** day's `days`. Apply the same rule to
   `[[budget.schedule]]`, which shares `in_window`, or split the two deliberately.
2. **The reported end is when quiet actually ends.** If periods touch or overlap, follow them to
   the last end (e.g. 21:30–00:00 then 00:00–06:00 reports "until 6:00 AM"). The existing split
   config then reads correctly too. The same goes for "locked until" and the advisor refusal text.
3. **Docs:** replace the two-block example in roles-and-config with one overnight block, and
   state the rule.
4. **Migrating existing configs.** This changes what an existing `end < start` period matches:
   a `sun` 23:00–07:00 period used to cover Monday morning only if `mon` was listed, and now does
   anyway. `00:00`-bounded split periods mean the same under both rules. Note the change in the
   release, per the migration rule ([[project-migrations-one-command-applies-pending-bridle-upgrad-xebc|xebc]]).
   `~/.bridle/config.toml` is the human's file, which agents may not edit, so the human updates
   their own config.

With it, the human's config can be:

```toml
[[focus]]
name  = "weeknight-sleep"
days  = ["sun", "mon", "tue", "wed", "thu"]
start = "21:30"
end   = "06:00"
mode  = "quiet"

[[focus]]
name  = "weekend-sleep"
days  = ["fri", "sat"]
start = "23:30"
end   = "06:00"
mode  = "quiet"
```

## Follow-up (the human, 2026-10-02)

The human, verbatim (via the advisor), after asking whether the fix had landed:

> It's likely fine as-is, but I would also accept a solution where the end has to be explicit such
> as +1 or +1d or something.
>
> Also whoever reads these blocks should union them all together and use that result to indicate
> how long the block runs. So that way any contiguous blocks would be clearly described and
> accurately.

The advisor checked br-14d6 as landed (761d545). The implicit rule (`end < start` means the next
day) stays; the human accepts it, so an explicit `+1d` isn't needed. `focus_end`
(`crates/bridle-daemon/src/config.rs`) gives the union's end. From the matching period it follows
any period of the same mode that matches at each end instant, so touching and overlapping blocks,
on any day, report one end (capped at 7 days). Blocks of different modes (`quiet` then `locked`)
aren't joined. The notice still names only the first block (`QUIET HOURS (<name>)`).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
