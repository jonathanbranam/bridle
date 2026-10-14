---
id: br-xcb4
title: "daemon rebuild can't parse task files whose thread comments contain ### or ## headings"
kind: bug
created: 2026-10-14
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-nbgp]
theme: reliability
resolved: 2026-10-14T19:08:41Z
---

## The ask

Reported by meta-notes' aide on vesta, 2026-10-14, for the human ("file bugs on this"). `bridle daemon rebuild --project meta-notes` failed with `error: internal: parsing task file: malformed thread`. The daemon itself wrote these task files, but several thread comments are agents' markdown reports containing their own headings, which the parser reads as thread entry headers (`### <kind> . <author> . <time>`): tasks/mn-3p28.md (`## Implementation complete`, `### Changes made`, ...), tasks/mn-pqwv.md (`### Outlook ...`), tasks/mn-4fsj.md (`## Implementation complete`, `## Completed with corrections`). Workaround in progress: demoting those headings to #### on meta-notes' bridle/state.

1. Whatever the daemon can write, rebuild must read back. Either escape headings in comment bodies when writing a thread, or make the parser treat only lines matching the exact entry-header shape as entry starts. A comment body is free markdown.
2. (Merged from br-s4pb; the human: "the error should name the file that failed".) A parse error must name the file, the line and the offending line's text, and rebuild should report every bad file in one pass, not stop at the first.

