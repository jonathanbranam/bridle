+++
id = "br-y7ht"
title = "[after daemon upgrades] Mint peer tokens so cross-project messaging works (stopgap until n63z)"
kind = "chore"
state = "claimed"
created_at = "2026-10-06T21:34:38.945Z"
updated_at = "2026-10-06T21:34:38.984395Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "human",
]
priority = "high"
priority_at = "2026-10-06T21:34:38.982153Z"
+++

Why: since br-3haz, 'bridle send --project <other>' goes through your own daemon's outbox, which forwards with a peer token. There are none, so cross-project messages fail (br-ubdc).

First: all three daemons must run br-3haz's code. bridle's started 14 s before br-3haz landed; bridle-ui's and track-web's are from Oct 4. The orchestrator is upgrading them ('bridle daemon restart --upgrade'), and the token command needs the new daemon.

Then, on dalek ([machine] name = dalek):
  bridle token create --peer dalek --project bridle
  bridle token create --peer dalek --project bridle-ui
  bridle token create --peer dalek --project track-web

Each prints a token once. Add them to ~/.bridle/credentials.toml:
  [peer]
  bridle = "<token from the first>"
  bridle-ui = "<token from the second>"
  track-web = "<token from the third>"

Check: BRIDLE_PROJECT=bridle-ui bridle send --project bridle external:orchestrator "peer test" should print 'queued o-...' and arrive in bridle.

## Thread

### note · external:orchestrator · 2026-10-06T21:34:38.982Z
created for the human, priority high

### note · external:orchestrator · 2026-10-06T21:34:38.984Z
To-do for you (high priority): [after daemon upgrades] Mint peer tokens so cross-project messaging works (stopgap until n63z). Finish it with `bridle task done br-y7ht`.
