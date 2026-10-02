---
id: v8kn
title: "Everything readable and editable through the daemons: file-backed records over the API"
kind: feature
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy, other-projects-submit-bridle-tickets-directly-for-triage-93xm, overnight-periods-must-say-1d-and-bridle-doctor-validates-th-r5s3]
tasks: [br-aca1]
---

## The ask

The human, verbatim (2026-10-02, via the advisor), choosing option B for the web UI (a
separate UI server in its own repo, [[a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]]):

> I prefer B - but that raises a unique problem that may be interesting to solve - how is work
> scheduled between projects that have inter-dependencies?
>
> I would take a very hard line on the API - it's fine to version it, but only one previous
> version maintained or NONE. A version is good regardless in the API, but bridle is changing
> rapidly and I own all parts so we don't need to support any other clients or users.
>
> I would plan to collocate the bridle and UI projects and have the advisor and orchestrators
> collaborate to make changes nearly simultaneously.
>
> Something missing here though in a separate UI is that I want to be able to read and edit or
> comment on literally everything bridle eventually - tasks, tickets, roadmaps, config, messages,
> incidents, etc. eventually everything is visible in the UI organized by machine and project.
>
> That goes beyond what bridle serves today since some of that is stored only as files on disk.

## What it means

Eventually everything bridle knows about is visible in the UI, organized by machine and project,
and the human can read, edit and comment on it: tasks, tickets, roadmaps, config, messages,
incidents, and the rest. The UI is a separate program, so it must get all of this through the
daemons' API.

## Today (advisor, checked 2026-10-02)

The daemons serve tasks, messages, events, agents, incidents (as tasks), the queue and usage.
Tickets (`docs/tickets/`), design docs and roadmaps (`docs/`), and config (`.bridle/config.toml`,
`~/.bridle/config.toml`) are files on disk that only the CLI or an agent in the clone reads.

## Advisor's recommendation

- **Each daemon is the one way in to its project's records**, files included: read endpoints for
  tickets, docs and config first, then writes. The UI never reads a clone's files. It may run on
  another machine, and going through the daemon keeps validation in one place (e.g. r5s3's config
  checks, `bridle ticket check`).
- **A write to a file is a commit**, attributed to the principal (the human through the UI),
  made where the daemon won't trip over agents' uncommitted work (m-2035 was a landing blocked by
  an uncommitted config edit).
- **Machine config** (`~/.bridle/config.toml`) isn't a project's; whichever daemon serves it, the
  rules stay: only the human edits it, never agents.
- Fits 93xm (submitting tickets over the API) and 7gk7 (bridle manages tickets).
- Order: read-only first, then ticket edits and comments, then config edits.
