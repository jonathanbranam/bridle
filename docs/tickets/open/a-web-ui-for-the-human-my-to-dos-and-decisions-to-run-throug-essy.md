---
id: essy
title: "A web UI for the human: my to-dos and decisions, to run through and check off"
kind: feature
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: [a-prototyper-role-in-the-base-workflow-build-only-from-the-p-6yb4]
see: [a-human-surface-beyond-the-cli-k4wq, answering-hitl-questions-from-mobile-u6wk, the-humans-to-do-list-and-restart-checklist-ex9q, tui-panels-and-seeing-the-work-y496]
tasks: [br-1665]
---

## The ask


The human, verbatim (2026-10-02, via the advisor; the full message is in
[[a-prototyper-role-in-the-base-workflow-build-only-from-the-p-6yb4|6yb4]]):

> I think we do need a web UI for Bridal for the human. In particular, I want to see tasks
> assigned to me, things that I need to do, decisions I need to make, and have those in a way I
> can easily run through them and check them off. The TUI is still really, really useful, so let's
> just keep it as well. But we need to start with a web UI for some of this, and next to that,
> similar to that, I think with within the same work, or as a sorry as as a dependency of that,
> before we start with that, I want to set up a prototype agent to be a standard agent that comes
> with the standard workflow pack.

## The ask

1. **A web UI for the human**, starting with what needs them: tasks assigned to them (the human
   to-dos, `--for-human`, ex9q), things to do, and decisions to make (questions to the human).
   It's laid out to run through quickly and check items off.
2. **The TUI stays.** The web UI is added beside it.
3. **Prototype first.** The prototyper role (6yb4) comes before this, and the UI starts as
   prototypes built with it.

## Notes (advisor)

- This answers the open question k4wq ("a human surface beyond the CLI?"), which named a local web
  board as a later option.
- Related: u6wk (answering questions from mobile; a web UI on the local network or behind HTTPS
  could serve that), y496 (TUI panels).

## Across projects and machines (the human, 2026-10-02)

The human, verbatim (via the advisor):

> So for the web UI, also, I really wanted to show tasks and to-dos across projects. It should be
> organized by project. Um, I still don't actually understand how the system for all of that
> works. But um, yeah, I only want one UI to load up for all my projects, if possible, even
> projects and other machines.

4. **One UI for every project**, including projects on other machines, organized by project.
   Tasks and to-dos from all of them in one place.

How it fits today (advisor, checked 2026-10-02): each project runs its own daemon on its own port.
A machine knows its local daemons from `~/.bridle/daemons/<project>.json` and its remote ones from
`~/.bridle/config.toml` (`[machines]`, and `[projects]` entries with `machine` and `port`, k7mw).
So a UI can list every project the way `bridle --project` finds them, and query each daemon's API.
The gaps:
- **Writes from another machine need a token there.** The human's token works only on the
  daemon's own machine ([[the-human-s-token-works-only-on-the-daemon-s-own-machine-3ehu|3ehu]]),
  so checking off a NUC to-do from dalek's UI needs 3ehu first. Reads from another machine need
  a token too; only loopback reads go without one.
- **Where the UI runs:** one process (e.g. `bridle ui`, or served by one daemon) that fans out to
  every daemon, rather than a UI per daemon. To settle in the design and the prototypes.

## Design

[[docs/design/human-web-ui|The human web UI]] (design only, for the human's review): screens, fan-out across
projects and machines, a separate `bridle ui` process, loopback-first auth, a server-rendered
stack, build tasks.

## The UI is a separate program (the human, 2026-10-02)

The advisor laid out four options: A, a browser app calling the daemons directly; B, a separate UI
server in its own repo; C, a `bridle-ui` crate in bridle's workspace; D, served by a daemon. The
human, verbatim (via the advisor):

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

Decided:

5. **Option B: a separate UI server in its own repo**, not part of the `bridle` binary or
   workspace. It reads the daemon list and the human's credentials, calls the daemons over HTTP,
   and serves the page. The browser never holds a bridle token. Reachable daemons are queried;
   defined but unreachable daemons and projects are reported as such. Several UIs may run
   anywhere that can reach the daemons.
6. **The API is versioned, with at most one previous version kept**, or none. Bridle changes
   fast and the human owns every client, so there's no wider compatibility to keep.
7. **The bridle and UI repos sit side by side**, and the advisor and orchestrators work on both so
   paired changes land nearly together. Scheduling across the two:
   [[scheduling-work-across-projects-that-depend-on-each-other-ztss|ztss]].
8. **Eventually everything is in the UI**, organized by machine and project, readable,
   editable and commentable: tasks, tickets, roadmaps, config, messages, incidents. Much of it is
   files on disk today:
   [[everything-readable-and-editable-through-the-daemons-file-ba-v8kn|v8kn]].

**The design doc needs revising:** `docs/design/human-web-ui.md` (br-761a, 82df110) recommends
`bridle ui` as a Rust crate in bridle's workspace (option C) and says v1 needs no new daemon
endpoints. Decision 5 replaces that. What bridle then provides: generated API types and a version
(decision 6), a machine-readable daemon list (e.g. `bridle projects --json`), and 3ehu for the
human's token across machines.
