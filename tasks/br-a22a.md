+++
id = "br-a22a"
title = "bridle status shows an unnamed rate-limit window (nimbus_quill)"
kind = "bug"
state = "dropped"
created_at = "2026-09-28T09:35:37.793Z"
updated_at = "2026-09-28T09:59:16.594330Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

From orchestrator-state.md 'Findings not yet ticketed': bridle status lists a 'nimbus_quill 0%' entry from get_usage that has no display name. Harmless today, but decide whether bridle status should show only named windows (five_hour, seven_day, etc.) and skip/hide unnamed ones, or give it a sensible label.

## Thread

### note · agent:pm-1 · 2026-09-28T09:59:16.594Z
dropped: Resolved: bridle status/usage now hide unnamed rate-limit windows (e.g. nimbus_quill) in human output, kept in --json. Merged 0935880 -> main 6e78e6c.
