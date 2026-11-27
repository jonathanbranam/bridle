+++
id = "br-kf7a"
title = "Concurrent self-upgrade on one machine: one daemon fails 'installing over ~/.cargo/bin/bridle: No such file'"
kind = "bug"
state = "integrated"
created_at = "2026-11-27T15:09:51.749Z"
updated_at = "2026-11-27T15:50:56.342504Z"
created_by = "external:orchestrator@vesta"
watchers = [
    "external:orchestrator@vesta",
    "agent:wkf7a",
]
branch = "bridle/wkf7a"
commit = "2406b3fb71b586e6450c885c7dad29137dfaeeaa"
summary = """
Cause confirmed: release::install_signed staged to one fixed <exe>.new, so one daemon renamed another's file away (ENOENT). Now every staged copy (release install, rollback restore, which had the same problem with <exe>.rollback) goes through signing::install_with, which uses a unique temp dir; signing::install (macOS and commit builds) already did. The release stash, install and UI steps run under a lock on .bridle.lock beside the binary (the UI's .tmp/.old names are shared too). The installer keeps the old binary as .bridle.prev and records tag + SHA256 in .bridle.installed; a daemon that finds a matching record installs nothing, takes .bridle.prev as its rollback copy and goes on to restart. upgrade_failed and rolled_back texts now read "upgrade (<project>): ...". Signing rules unchanged (identifier bridle, signed copy renamed in).

Caveats: the first upgrade into this version still runs the old install code. Commit-build mode is not under the new lock (its install was already collision-free and only bridle's own daemon uses it). Not fixed, found while reading: workflow_checkout::ensure also uses a shared scratch name (~/.bridle/workflow/.fetching-<tag>) and can fail the same way when daemons fetch at once (non-fatal, retried at next start)."""
ticket = "kf7a"
+++

docs/tickets/open/upgrade-race-kf7a.md

submitted by external:orchestrator@vesta

## What happened

On vesta, 2026-11-27 15:08:59-15:09:10 UTC, every project daemon self-upgraded to release v0.8.1 at the same time. The forwarded system messages to external:orchestrator@vesta-home included one:

    [upgrade_failed] upgrade: release v0.8.1 not installed; the daemon is unchanged: installing over /home/jbranam/.cargo/bin/bridle: No such file or directory (os error 2)

The other daemons installed it, and ~/.cargo/bin/bridle is v0.8.1 (mtime 15:09 UTC). All daemons restarted at v0.8.1 and resumed their agents. The forwarded message did not say which project's daemon failed.

## Likely cause

Several daemons on one machine install over the same shared binary path at once. One daemon's replace (a rename or unlink) races another, so the path briefly doesn't exist. The race was harmless this time, but the failing daemon reports a failure that didn't matter, and on a different timing a daemon might keep running the old version.

## The ask

Make self-upgrade safe when several daemons share one binary: either one install per machine (a lock, or skip when the installed binary already matches the target version), or treat "already at target version" as success. Include the project name in the upgrade_failed message.

Filed by orchestrator (vesta-home).

## Thread

### note · external:orchestrator@vesta · 2026-11-27T15:09:51.750Z
submitted by external:orchestrator@vesta

### note · external:orchestrator · 2026-11-27T15:10:31.029Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-27T15:10:31.160Z
Readied by orchestrator (dalek): bug, self-evident fix (rule ready-without-asking).

### note · agent:wkf7a · 2026-11-27T15:34:58.967Z
wkf7a: fix committed on bridle/wkf7a (main merged), 22 targeted tests pass; full 'just check' running in the background. Next: summary, push, built, report to manager-2. Keeping the claim.

### note · agent:wkf7a · 2026-11-27T15:50:47.023Z
built: eb2cf2c1 (branch bridle/wkf7a)

### note · agent:wkf7a · 2026-11-27T15:50:47.065Z
done: concurrent self-upgrade safe, upgrade_failed names the project; eb2cf2c1; env -i just check exit 0, 2044 tests passed, 6 skipped. Not fixed (separate ticket?): workflow_checkout::ensure shares .fetching-<tag> between daemons.

### note · agent:manager-2 · 2026-11-27T15:50:56.178Z
integrated: 2406b3fb71b586e6450c885c7dad29137dfaeeaa (branch bridle/wkf7a)

### note · agent:manager-2 · 2026-11-27T15:50:56.342Z
ticket file moved to docs/tickets/resolved/upgrade-race-kf7a.md
