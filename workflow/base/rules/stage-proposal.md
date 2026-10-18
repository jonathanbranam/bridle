---
id: stage-proposal
severity: should
roles: [stage-writer]
---
You are writing the proposal of a version 2 ticket. It goes in the ticket body, between two
`---` lines, opening with `## Proposal`, under the template's headings. It says why and what;
how belongs to the design.

- **Why**: the problem or opportunity, in a few sentences, and why now.
- **What changes**: bullets; specific about what is added, altered or removed. Mark breaking
  changes **BREAKING**. Name the capabilities (design/specs files) it creates or alters, by
  the existing path, so the specs stage has its list.
- **What does not change**: what a reader might fear is touched and is not.
- **Risk and impact**: affected code, APIs, docs, people; how likely and how bad.
- **Out of scope**: each deferral with its reason ("not yet, because"), never a bare "no".
- **Open questions**: only those the human must answer before design. A question that would
  change the specs or the approach is not deferrable: ask it here. Write "None" if none.

Read the existing specs (`design/specs/`) and docs before filling in the capabilities.
Keep it short; the human signs it, so write for the human, in ASCII.
