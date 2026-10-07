---
id: hesj
title: "Focus override: accept local time in until, like the [[focus]] config"
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [focus-hours-quiet-and-locked-cvaq]
kind: feature
---

## The ask


The human, verbatim (2026-10-01, via the advisor), after being told the override's `until` is UTC:

> ok, UTC sucks, file a low-prio small ticket to support local timezone same as in the config.

## Today

`~/.bridle/focus-override.toml`'s `until` must carry an offset or `Z`: `parse_override` in
`crates/bridle-daemon/src/focus.rs` parses it as `DateTime<Utc>` and rejects anything else
(the file is then logged as malformed and ignored). The `[[focus]]` periods in
`~/.bridle/config.toml` are already local wall-clock times (`start = "08:00"`, checked against
`DateTime<Local>`). [[docs/design/agent-host/roles-and-config|roles and config]] documents
`until` as UTC.

## The change

Accept a local time in `until`, read in the machine's local zone like the `[[focus]]` config:
a TOML local datetime (`until = 2026-10-01T22:00:00`) and probably a bare local time
(`until = 22:00`, meaning the next 22:00). Keep accepting the offset/`Z` forms. Update the
roles-and-config section and its example. Small; low priority.
