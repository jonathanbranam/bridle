+++
id = "br-crht"
title = "One cross-platform process-table read: sysinfo + getpgid on Linux too, drop the /proc parser"
kind = "chore"
state = "pending"
created_at = "2026-10-09T01:14:41.562Z"
updated_at = "2026-10-09T01:14:41.563578Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:orchestrator",
]
parent = "br-9z2n"
+++

Follow-up to br-9z2n (landed 018b9cfd). The human approved this via aide (2026-10-08 ~9:25 PM ET): "Yeah, let's fix it. Yes, definitely do that" -- use one cross-platform path (sysinfo + nix getpgid) instead of /proc on Linux and sysinfo on macOS. Simplifies the code and tests one path on every OS, including WSL2.

Change (crates/bridle-daemon/src/containment.rs):
- Remove the cfg(target_os = "linux") /proc/<pid>/stat reader and its parser tests; make the sysinfo (pid, ppid, start) + getpgid path the only native path on all OSes. Keep the ps fallback on error or empty result.
- Start-time compatibility: br-9z2n tags native starts "n:" and is_same_process checks an untagged (ps-format) start against ps. The Linux /proc "n:" starts stored by daemons running 018b9cfd have a different value than sysinfo's start for the same process. Use a different tag for the sysinfo format (e.g. "s:") and make is_same_process handle all three stored forms across an upgrade: untagged -> compare via ps; "n:" (the removed /proc format) -> compare via ps too, or treat as "cannot tell, adopt as before" -- whichever matches what an untagged one does today (read the function and its tests; the rule is: never compare two different formats, and never signal a reused pid). Add a test per stored form.
- If sysinfo does not report a usable start time or ppid on Linux for some processes (permissions, zombies), the ps fallback or skipping that process must not make descendants()/reap miss a live child of an agent; check with the existing spawned-child test.

Files: crates/bridle-daemon/src/containment.rs, docs/design/agent-host/agents.md (Containment: one path), CHANGELOG.md. Cargo.toml only if sysinfo features need changing.

Acceptance: just check passes (on this Mac; Linux is not testable here, so keep the code OS-independent and say so on the thread). Existing containment tests pass; own-pid/parent_id and spawned-child tests pass; new stored-form tests. Sonnet.

Migration: none for files; the stored-start compatibility above is the upgrade story (pid files written by the previous daemon are still adopted correctly after the swap).

Out of scope: other ps callers, the tracker interval, load measurement.
