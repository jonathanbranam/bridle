+++
id = "br-4fpk"
title = "Rule set and write-time guard: no file paths that collide on a case-insensitive disk (mu-bgb6)"
kind = "feature"
state = "planned"
created_at = "2026-11-09T14:59:03.928Z"
updated_at = "2026-11-09T15:14:03.857215Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "4fpk"
+++

docs/tickets/open/rule-set-and-write-time-guard-no-file-paths-that-collide-on-4fpk.md

## Thread

### note · external:aide · 2026-11-09T14:59:06.833Z
readied by external:aide

### note · agent:pm-1 · 2026-11-09T14:59:51.345Z
pm-1 brief: the human asked for this directly and wants it worked (see the ask), so it is planned without another design round. Split in two so each fits one worker; this ticket is SLICE 1, slice 2 is a new ticket blocked-by this one (they share the rules/hooks files).

SLICE 1 (this ticket), model Sonnet: a base rule plus a write-time guard.
- Base rule `workflow/base/rules/no-case-colliding-paths.md` (roles: all that write files; follow an existing rule file for the front matter, and put the Why from the ask: meta-notes-ui, TimeLog.tsx vs timelog.ts, the human's words). It says: never create a path that equals another path in the same directory when lowercased (with and without the extension stripped for the same-stem form), because macOS and Windows checkouts collide while Linux CI hides it.
- A PreToolUse hook command in the style of `bridle kill-guard` (find it, copy its wiring and how agent settings register hooks): `bridle case-guard` refuses a Write (and Edit/NotebookEdit creating a file) whose target collides, case-insensitively, with an existing tracked or on-disk sibling (same directory, equal lowercased name; also equal lowercased stem for a different extension, because bundlers pick the wrong one). The error names both paths and the fix. Allow writing the file that already exists under the same exact name.
- Register the hook for the agent roles that write files, wherever kill-guard is registered.
Tests: guard refuses Foo.md vs foo.md, refuses TimeLog.tsx vs timelog.ts, allows an exact rewrite and unrelated names. Docs: docs/design (hooks section) and CHANGELOG. Existing-project impact: the hook is in the bridle binary and base rules, so projects get it on upgrade; no project-file migration is needed (say so in the CHANGELOG). Acceptance: `just check`.
Out of scope: the typescript-pack naming convention and the check-command backstop (slice 2), renaming any existing project's files.
