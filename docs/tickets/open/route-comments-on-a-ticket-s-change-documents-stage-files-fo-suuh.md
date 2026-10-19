---
id: br-suuh
title: Route comments on a ticket's change documents (stage files) for the stages, not to a generic document reviewer
kind: feature
created: 2026-10-19
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-2ae8, br-ewk8, br-tj7q, br-ggbx, br-re79]
hold: design
---

## The ask

Found in the human's hands-on try (br-ewk8, ticket br-2ae8), reviewing the design stage.

The human, 2026-10-19, to advisor tj7q: "We should have a ticket I think for v1 that comments are routed specifically for change documents. We need to walk thru what that means exactly later."

Today a version 2 ticket's stage files (`docs/tickets/open/<ticket>/design.md`, `specs.md`, `tasks.md`) are ordinary documents to the review watcher. A comment the human leaves in one from the web UI goes, after `[review] quiet_minutes`, to a generic `document-reviewer` agent (`doc-design-<hash>`), which knows nothing of stages: it doesn't sign, doesn't send the stage back, and doesn't follow the stage rules. The stage writers see the human's words only through ticket thread comments (`ticket next --comments`), and the specs writer never reads `design.md` at all.

The ask (to be walked through with the human before any design): comments on a change document (a ticket's stage files, maybe the proposal too) are routed for that purpose, to the stage's writer or the next stages', rather than to a generic document reviewer. What "routed specifically" means is the open question for the human.

hold: design. Don't design or build until the human has walked through it.
