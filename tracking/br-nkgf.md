+++
id = "br-nkgf"
title = "Context-size notices (150k) for one session are sent to other sessions too, so the human hears them from several agents"
kind = "bug"
state = "planned"
created_at = "2026-11-21T16:21:51.380Z"
updated_at = "2026-11-21T16:35:10.072280Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
ticket = "nkgf"
+++

docs/tickets/open/context-notice-fanout-nkgf.md

submitted by external:aide@vesta

## The human's words (2026-11-21 ~11:21 AM ET, via the notes aide)

"Yeah, this is another weird thing with Bridle. Do those notifications get sent to every agent, like on the machine or on the project? I'm not sure. I keep getting told this from multiple agents when so-and-so is at 150k of context. That is a bug."

## What happened

On the notes daemon (vesta), the aide (external:aide) received m-0366 from system: "advisor/agentic-dev is at 151k tokens of context. `bridle session restart advisor/agentic-dev --fresh` restarts it clean; `--handover` asks for a note first." The advisor is a different session on the same project (notes). The aide relayed it to the human, as other agents also do, so the human hears the same notice several times. (Earlier, m-0353 to the aide about its own 150k context said "the human has been told", so the human also gets it directly.)

## The ask

A session's context notice goes to that session (and the human once), not to other agents on the project or machine. Find who else receives these notices (other aides, advisors, orchestrators) and stop the fan-out.

Filed by aide (notes).

## Thread

### note · external:aide@vesta · 2026-11-21T16:21:51.381Z
submitted by external:aide@vesta

### note · external:orchestrator · 2026-11-21T16:22:25.953Z
readied by external:orchestrator
