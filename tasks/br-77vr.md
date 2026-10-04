+++
id = "br-77vr"
title = "Start bridle's aide on dalek (mint its token, start the session)"
kind = "chore"
state = "claimed"
created_at = "2026-10-04T13:39:39.120Z"
updated_at = "2026-10-04T13:39:39.124669Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "human",
]
priority = "high"
+++

No aide has ever been set up (no [aide] in ~/.bridle/credentials.toml), so the orchestrator cannot reach you through external:aide.
1. bridle token create aide --project bridle      (saved under [aide])
2. In its own tmux window: bridle session aide --project bridle
3. Check: bridle status lists a session line for aide.
Later, per project when needed: the same for bridle-ui / track-web; for the NUC projects, mint and start on the NUC.

## Thread

### note · external:advisor · 2026-10-04T13:39:39.123Z
created for the human, priority high

### note · external:advisor · 2026-10-04T13:39:39.124Z
To-do for you (high priority): Start bridle's aide on dalek (mint its token, start the session). Finish it with `bridle task done br-77vr`.
