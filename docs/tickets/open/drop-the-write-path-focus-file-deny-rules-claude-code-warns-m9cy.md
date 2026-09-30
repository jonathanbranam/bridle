---
id: m9cy
title: "Drop the Write(path) focus-file deny rules: Claude Code warns on every session start"
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq]
---

## The ask


The human, 2026-09-30:

> Getting this error on named Aadvisor startup:  bridle session advisor gnar
> Permission deny rule (/tmp/claude-501/claude-settings-8d32cc8e07967fdd.json): Write(~/.bridle/focus*) is not matched by file permission checks — only Edit(path) rules are. Use Edit(~/.bridle/focus*) instead (Edit rules cover all file-editing tools).
> Permission deny rule (/tmp/claude-501/claude-settings-8d32cc8e07967fdd.json): Write(~/.bridle/config.toml) is not matched by file permission checks — only Edit(path) rules are. Use Edit(~/.bridle/config.toml) instead (Edit rules cover all file-editing tools).

## What's there now

717a444 (cvaq) added `Write(~/.bridle/focus*)` and `Write(~/.bridle/config.toml)` next to their
`Edit(...)` twins, in `DENY_FOCUS_FILES` (`crates/bridle-daemon/src/config.rs`, "`Edit` rules
cover `Write` too; both are listed to be explicit") and in the session launcher's `LEAN`
settings (`crates/bridle/src/session.rs`). Claude Code 2.1.286 warns about each `Write(path)`
rule at startup. The `Edit(...)` rules already cover Write, so the denial still holds and only
the warning is new.
