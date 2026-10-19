---
id: br-ggbx
title: "Relay the human's comments through an agent: approve with a comment, signed by the human via the agent"
kind: fix
created: 2026-10-19
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-2ae8, br-ewk8, br-tj7q, br-bdwh]
---

## The ask

Found in the human's hands-on try (br-ewk8, ticket br-2ae8), reviewing the design stage.

The human, 2026-10-19, to advisor tj7q: "Hmmm. Yes I should be able to relay comments through an agent. That needs to be possible. I want to answer you: 'approve option 1 with caveat....' or whatever and that should be signed by me via you"

Today:
- `bridle ticket approve <id> <stage>` run by an agent records "signed by human via <principal>", but it takes no text.
- `bridle ticket comment` run by an agent is authored by that agent. `ticket next --comments` (`crates/bridle/src/ticket.rs`, `next_cmd`) counts only thread entries whose `from` contains "human", so a comment an agent relays for the human never reaches the stage writers, whatever its text says.

The ask:
- `bridle ticket approve <id> <stage>` takes the human's words with the signature (e.g. `--comment "<text>"` / `--comment-file`). It is recorded as the human's, via the agent that ran it ("human via external:advisor/tj7q"), in the same thread note as the signature or right beside it.
- `bridle ticket comment` can likewise record a comment as the human's via the agent (e.g. `--for-human`), for comments without a signature. `ticket changes` already carries text; it is recorded the same way when an agent runs it for the human.
- `ticket next --comments` counts these as the human's, so the next stage writers (and a redo) get them.
- Who may do it: the roles that relay for the human (advisor, aide, orchestrator). A worker or designer may not.
- Update `docs/design/stages.md` ("Marks, signing and claims", and the `--comments` paragraph) and `docs/design/cli.md`.
