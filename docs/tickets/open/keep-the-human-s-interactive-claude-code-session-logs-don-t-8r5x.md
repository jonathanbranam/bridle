---
id: 8r5x
title: Keep the human's interactive Claude Code session logs; don't let them be purged
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [34wz, jttf]
tasks: [br-2b0b]
---

## The ask


Record, don't build yet. The human, verbatim (2026-10-03, via the advisor, a list headed "Ideas to record not build"):

> Save my Claude session logs from interactive sessions. Don't let them be purged

## Notes

- Claude Code keeps session transcripts under `~/.claude/projects/` and deletes old ones after
  `cleanupPeriodDays` (default 30). Neither `~/.claude/settings.json` nor this repo's
  `.claude/settings.json` sets it today (checked 2026-10-03). Options: raise it, or copy
  interactive transcripts (orchestrator, advisors) somewhere bridle keeps.
- Not the same as [[event-and-transcript-retention-34wz|34wz]], which is about bridle's own
  transcripts growing without bound.
