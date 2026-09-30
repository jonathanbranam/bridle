---
id: 8awb
title: Short product briefs of how each part of bridle works today
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [docs-kept-current-as-part-of-every-change-3ndf, which-docs-live-in-bridle-and-which-in-markdown-hv8e]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> The orchestrator is suggesting migrating a project, maybe meta notes, onto Bridal Specs, but
> I haven't read about how Bridal Specs work yet. [...] we need a certain level of
> documentation that I can use to understand how things are working.
>
> I want a solution that publishes some documentation that I can read, like a product brief,
> to understand the different parts of Bridal and how it's working today. I think we're
> publishing a change log. I haven't checked that yet, but this would be a way for me to go in
> and see what's the latest with Bridal specs: how they work and what the current behavior is.
>
> It doesn't need to be super detailed, but it should be enough that I can read it and
> understand.
>
> The way I would use that right now is I want to understand how Bridal specs work, what's
> currently implemented, and not just how the CLI works, but how the system works around it.
> I'm interested in doing the same thing for the task system and other parts of Bridal. I
> think that would be useful. Again, brief: don't get super detailed on it because we're still
> very much in early development.

("Bridal" is bridle, from dictation.)

## What exists today (advisor, 2026-09-29, at f5cd08b)

- `CHANGELOG.md` exists and is kept per task (one line per change, with the task id). It says
  what changed, not how a part works.
- `docs/design/agent-host/` describes the daemon and agent host as built. The rest of
  `docs/design/` (specs, spec flow, tasks, workflow layers, ...) is written as design, much of
  it ahead of or different from what's built. Nothing says which parts are built.
- `docs/design/cli.md` covers the commands, not the system around them.
- Nothing is written for the human as a reader: no as-built overview per part.

## Shape (advisor's reading of the ask; for the human to confirm)

- One short brief per part of bridle, as built today: what it's for, how it works end to end
  (not only the CLI), what's implemented and what isn't yet. A page or two each.
- First: specs (what the human needs now, before deciding on a specs migration), then the
  task system (tasks, queue, landing). Later: the agent host, workflow layers, and others.
- Published somewhere the human reads comfortably, linked from one index.
- Kept current by the rule in [[docs-kept-current-as-part-of-every-change-3ndf|docs kept current]].

## Open

- Where to publish. The repo's markdown on GitHub (e.g. `docs/briefs/`) is the simplest. A
  rendered page (e.g. an Artifact) reads better on a phone but needs republishing.
