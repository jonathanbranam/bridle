+++
id = "br-xcb4"
title = "daemon rebuild can't parse task files whose thread comments contain ### or ## headings"
kind = "bug"
state = "claimed"
created_at = "2026-10-14T11:15:21.015Z"
updated_at = "2026-10-14T18:43:50.665675Z"
created_by = "external:aide@vesta"
watchers = [
    "external:aide@vesta",
    "agent:xcb4",
]
summary = "Thread parser (state_branch.rs parse_thread) is now line based: only a line of exactly the writer's entry-header shape (### known-kind, author, RFC 3339 time, joined by middle dots) starts an entry, so comment bodies with ## / ### headings read back and existing files need no migration. Parse errors give line number and text; thread/task errors from list_tasks (rebuild) are prefixed with the file path and collected, so every bad file is reported in one pass. list_tasks now also fails on an unparseable tickets/<id>.md (before, overlay_thread silently dropped it, which is the empty 'ticket show' thread of br-5cru); read_task still tolerates it. Tests: heading round trip, bad-line error, two bad files. Docs: storage.md, CHANGELOG. Caveat: a body with a literal '## Thread' line still confuses the head/thread split."
ticket = "xcb4"
+++

## The ask

Reported by meta-notes' aide on vesta, 2026-10-14, for the human ("file bugs on this"). `bridle daemon rebuild --project meta-notes` failed with `error: internal: parsing task file: malformed thread`. The daemon itself wrote these task files, but several thread comments are agents' markdown reports containing their own headings, which the parser reads as thread entry headers (`### <kind> . <author> . <time>`): tasks/mn-3p28.md (`## Implementation complete`, `### Changes made`, ...), tasks/mn-pqwv.md (`### Outlook ...`), tasks/mn-4fsj.md (`## Implementation complete`, `## Completed with corrections`). Workaround in progress: demoting those headings to #### on meta-notes' bridle/state.

1. Whatever the daemon can write, rebuild must read back. Either escape headings in comment bodies when writing a thread, or make the parser treat only lines matching the exact entry-header shape as entry starts. A comment body is free markdown.
2. (Merged from br-s4pb; the human: "the error should name the file that failed".) A parse error must name the file, the line and the offending line's text, and rebuild should report every bad file in one pass, not stop at the first.

## Thread

### note · external:aide@vesta · 2026-10-14T11:15:21.016Z
submitted by external:aide@vesta

### note · agent:pm-1 · 2026-10-14T11:15:34.570Z
Accepted by pm-1 (br-s4pb merged in as ask item 2). Brief. Model: Sonnet. Size: S.

Find the thread parser: git grep -n "malformed thread" crates. Prefer making the parser accept only exact entry-header lines (the shape the writer emits, in the task-file writer next to it) as entry starts, so existing files with headings in comments read back without rewriting; add a write-side guard only if the parser cannot be unambiguous (then migrate on read, no manual file edits). Error messages: every parse failure names file path, line number and line text; rebuild collects all bad files and reports them together. Tests: comment body with ## and ### lines round-trips; bad file error names file and line; two bad files both reported. Docs: the task-file format section (git grep -n "thread" docs/design/storage.md), CHANGELOG.md. No migration. Acceptance: just check. Out of scope: other parsers.

### note · external:orchestrator · 2026-10-14T11:39:50.289Z
Another case (orchestrator): 'bridle ticket show br-5cru' prints an empty 'Thread (br-5cru, built):' though tracking/br-5cru.md on bridle/state holds the worker's full design comment, which has ### headings. The product-manager advisor concluded the design was missing because of it.

### note · agent:xcb4 · 2026-10-14T18:43:50.665Z
done: thread parser accepts only exact entry-header lines; parse errors name file/line/text; rebuild reports all bad files; just check exit 0, 1676 tests passed; 8635a946 (main merged in; summary written)
