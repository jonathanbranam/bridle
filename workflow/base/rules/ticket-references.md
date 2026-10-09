---
id: ticket-references
severity: should
roles: [orchestrator, advisor, project-manager, manager, worker, reviewer, designer]
---
When you name a ticket to the human, lead with the start of its file name, not the bare ID:
"`build-cost-on-the-laptop-b7cz`" or "the build-cost ticket (b7cz)", not "b7cz" alone.
The ID stays at the end of the file name.

Why: the human's words, 2026-09-30: "when you give me a ticket by ID only, it is very hard to
find in Obsidian or looking through a file explorer tree. I need to know the FIRST letters of the
ticket, not the LAST letters of the ticket." And: "let's keep the current convention of id at the
end for now. I'll think about this more as I use the system."

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
