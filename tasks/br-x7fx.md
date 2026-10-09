+++
id = "br-x7fx"
title = "Turn self_upgrade back on ([daemon] self_upgrade = true in bridle's .bridle/config.toml) once br-7ufd (batched self-upgrades) has landed"
kind = "chore"
state = "claimed"
created_at = "2026-10-09T22:04:36.827Z"
updated_at = "2026-10-09T22:04:36.832753Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "human",
]
priority_at = "2026-10-09T22:04:36.830434Z"
+++

From the human, 2026-10-09 ~6:30 PM ET, via aide: "I turned off self_upgrade for now as well; I assume that will be read after the next upgrade; so, I need to be reminded to turn it back on after the batching is implemented." The change is uncommitted in the human's checkout (/Volumes/Data/work/bridle/bridle/.bridle/config.toml); the daemon reads it only at start. When br-7ufd lands, set it back to true and restart the daemon (or let the next manual upgrade pick it up).

## Thread

### note · external:aide · 2026-10-09T22:04:36.830Z
created for the human, priority normal

### note · external:aide · 2026-10-09T22:04:36.832Z
To-do for you (normal priority): Turn self_upgrade back on ([daemon] self_upgrade = true in bridle's .bridle/config.toml) once br-7ufd (batched self-upgrades) has landed. Finish it with `bridle task done br-x7fx`.
