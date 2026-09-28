+++
id = "br-a22a"
title = "bridle status shows an unnamed rate-limit window (nimbus_quill)"
kind = "bug"
state = "open"
created_at = "2026-09-28T09:35:37.793Z"
updated_at = "2026-09-28T09:35:37.793Z"
+++

From orchestrator-state.md 'Findings not yet ticketed': bridle status lists a 'nimbus_quill 0%' entry from get_usage that has no display name. Harmless today, but decide whether bridle status should show only named windows (five_hour, seven_day, etc.) and skip/hide unnamed ones, or give it a sensible label.
