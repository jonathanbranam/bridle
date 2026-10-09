---
id: ascii-in-editable-text
severity: must
roles: [orchestrator, project-manager, manager, worker, reviewer, document-reviewer, advisor, designer]
---
Anything the human may type or edit (tickets, docs, comment threads, config) uses ASCII. No
U+00B7 middle dot, curly quotes, arrows or em dashes: a Vim user can't easily type them.

Why: the human's words, 2026-10-04: "As a general rule, avoid any Unicode symbols in any sort of
plain text place where I might be typing or editing. I think you're using a Unicode dot there. I
can't type that in Vim, at least not easily, and I don't want to, so just ASCII." (ticket ehv6)
