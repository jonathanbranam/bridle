---
id: br-vmrc
title: Mail digest stamps with the host's local zone but says US Eastern; fix is_bare_sent_notice's doc comment
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

From the orchestrator's verification of br-bft2. The digest header says US Eastern but stamps with chrono::Local, which is UTC on a UTC host. Either stamp in America/New_York for real (chrono-tz, if already a dependency; otherwise a fixed rule is not enough because of DST) or print the zone actually used in the header; prefer real Eastern, since the human's rule human-timezone says times shown to the human are Eastern. Also fix the doc comment on is_bare_sent_notice, which does not match its code (make the comment say what the code does, or fix the code if the comment states the intent: the intent is dropping items whose only text is "message sent" with no link). Files: crates/bridle-mail digest code, docs/design/mail.md. Tests: a UTC host still prints Eastern times. Acceptance: just check. Model: Sonnet.
