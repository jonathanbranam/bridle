---
id: r5s3
title: Overnight periods must say +1d, and bridle doctor validates the machine config
kind: feature
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [focus-hours-an-overnight-period-belongs-to-the-day-it-starts-3xr4, focus-hours-quiet-and-locked-cvaq, project-migrations-one-command-applies-pending-bridle-upgrad-xebc]
tasks: [br-1c2a]
---

## The ask


The advisor listed formats for an explicit next-day end, numbered 1–5. Option 5 was "strict mode":
`end = "06:00+1d"`, required whenever the end is past midnight, with `"06:00"` alone after a later
`start` an error. The human, verbatim (2026-10-02, via the advisor):

> I like 5 and I don't think it will break any current configs. It feels better and fixes the
> problem and avoids accidents. Do we have a command to validate config like bridle doctor or
> something? That would be useful to do at that time or any other config change time.

## Today (advisor, checked 2026-10-02)

- After 3xr4 (761d545), `end < start` silently means the next day, in `[[focus]]` and
  `[[budget.schedule]]` alike (`in_window`, `crates/bridle-daemon/src/config.rs`).
- The human's `~/.bridle/config.toml` has three blocks with `end < start`: focus
  `weekday-sleep` 21:30–00:00, focus `weekend-sleep` 23:30–00:00, and budget `night` 23:00–08:00.
  Its `low`/`burst` budget presets are `days = []`, 00:00–00:00.
- `bridle doctor` checks that the project's `.bridle/config.toml` loads, plus git, tools,
  `.gitignore`, ports and role prompts. It has no explicit check of the machine
  `~/.bridle/config.toml`, and nothing validates config when it's edited.

## The change

1. **Strict overnight ends.** A period whose end is past midnight writes it as `HH:MM+1d`
   (`end = "06:00+1d"`). An `end` before `start` without `+1d` is a config error naming the block
   and the fix ("night: end 08:00 is before start 23:00; write \"08:00+1d\""). Same for
   `[[focus]]` and `[[budget.schedule]]`.
2. **`end = "00:00"` is midnight at the end of the start day** and needs no `+1d`. The human's two focus blocks stay valid as written. Only budget `night`
   needs `08:00+1d`.
3. **`start == end` stays empty** (the `days = []` presets are untouched).
4. **`bridle doctor` validates the machine config:** a check that parses every section of
   `~/.bridle/config.toml` (`[[focus]]`, `[[budget.schedule]]`, the rest) and reports each bad
   block with its fix. Running `bridle doctor` is the check after any config change.
5. **Migration:** the budget `night` block is the human's file, which agents may not edit. The
   release notes say what to change, and `bridle doctor` names it. Decide what the daemon does
   with an invalid block before release: today a config that doesn't load fails loudly.

## Approved (the human, 2026-10-02)

The human, verbatim, on the summary of items 1–3 above (strict `+1d`, `00:00` as midnight of the
start day, `bridle doctor` checking the machine config): "Yes on 3. All of that looks good."
