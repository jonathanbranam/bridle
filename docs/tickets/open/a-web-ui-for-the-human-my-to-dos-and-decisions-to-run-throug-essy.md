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
   workspace. (Superseded by option F below, the same day.) It reads the daemon list and the human's credentials, calls the daemons over HTTP,
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

## Option F: a bridle gateway and a TypeScript UI (decided, the human, 2026-10-02)

The human asked to slow down and hear the case for option C first, then asked, verbatim (via the
advisor):

> is there any benefit in having a, a bridal API that lives within our project? So like, not
> something that doesn't serve the HTML or JavaScript, but just serves the UI and handles all the
> token and auth and everything. And then the UI would be like a separate project. You know,
> probably in TypeScript is what I would reach for naturally. I don't know, it, it feels like a
> stretch to put the HTML host in Rust.

After the advisor's answers (below), verbatim: "Yeah, no, I think this is a great solution. I feel
really good with it. Let's write this up and we can, I think, begin work on the gateway." This
replaces decision 5 and the design doc's option C.

- **`bridle gateway`**: a subcommand of the `bridle` binary, in bridle's workspace, run as its own
  process (not inside a daemon), one per machine where the human wants the UI, as a launchd or
  systemd service like the daemons. Separate because daemons are per project and restart often
  (self-upgrades); the gateway spans projects and must outlive those restarts. It also keeps UI
  load off the daemons.
- **What it does:** finds every defined daemon (local registry and `[projects]`/`[machines]`),
  fans out, and serves one API built for the UI, across projects and machines, with unreachable
  daemons and projects reported. It holds the human's tokens (per machine, 3ehu) and the browser
  login; the browser and the TypeScript code never hold a bridle token. It's where file-backed
  records arrive (v8kn). The TUI or the orchestrator could use it later.
- **Contract:** the gateway uses `bridle-api` like the CLI, so daemon API changes are compiler-
  checked in `just check`, and the daemon API stays internal. The gateway's API is the one the UI
  depends on: versioned, at most one previous version kept (decision 6), with TypeScript types
  generated from its Rust types (ts-rs; utoipa plus openapi-typescript if a full endpoint spec
  becomes worth it).
- **Login:** username and password, `argon2` hash in machine config, an `HttpOnly`,
  `SameSite=Strict` session cookie. Reached over Tailscale (encrypted; `tailscale serve` can add
  HTTPS). No external hosting planned. Alternative: trust Tailscale identity via `tailscale serve`.
- **The UI**: its own TypeScript repo beside bridle's. Its build (`dist/`) is installed into a
  folder the gateway serves (e.g. `~/.bridle/ui/`), so page and API share an origin. The build
  records the API version it targets; the gateway refuses or warns on a mismatch. Install for now
  by a script in the UI repo; later the automatic upgrade builds it too (ztss). In development the
  UI's dev server (Vite) proxies API calls to the gateway. Not compiled into the `bridle` binary
  (that would put Node in bridle's build and tie the releases).
- **Gateways don't talk to each other.** Each reaches the daemons its machine's config defines and
  its credentials allow. One gateway, e.g. on the NUC only, can reach every project if the NUC's
  config lists dalek's projects, holds the human's tokens for dalek, and dalek's daemons listen
  on its Tailscale address (k7mw). A sleeping laptop's projects show as unreachable.

`docs/design/human-web-ui.md` is to be revised to this before build tasks.

## Gateway v1: the human's answers (2026-10-02)

The human, verbatim (via the advisor), on the advisor's six questions:

> Agree on login, that's fine. I store all my passwords in 1Password anyway, and it automatically
> logs in, so it's no issue at all to me if I have to log in every time.
>
> It should use my token. I don't really see any need to record the fact that this came through
> the gateway or not. That doesn't make any difference. Yeah, just use my token.
>
> Yeah, the main thing for version one is my tasks or to-dos, the things I need to follow up on.
> Um, specifically, like, the orchestrator keeps telling me the things I need to do, and it's a
> waste of context for the orchestrator. They just scroll out of view when I'm busy, so they just
> scroll, scroll, scroll, scroll, scroll. There's a whole bunch of things that need my decision
> that I never go look at. And I can't find them. They're, it's just like not a useful or a very
> functional way for me to review what needs done. The biggest thing for this is that they need to
> be retractable. Um, I, I don't know if, for sure if that's done, but that'd be separate work, of
> course. But if your orchestrator asks me to do something and then another ticket comes along or
> another, you know, it gets resolved in some other way, then. should be able to retract that and
> we, we should have you know an audit trail that that happened but I don't want to see it anymore
> I don't want to show up with 20 messages that were resolved by some other process just because I
> didn't read them so um, that's a problem I think we already have with the inbox thing um, I'm
> not sure how to deal with that we'll, we'll leave that inbox aside for now but I, You know, like
> my Gmail, it grows forever because I, there's things I just don't care about. And it's, it takes
> me time and effort to go mark them off and get rid of them.
>
> The folder name bridal-ui is fine. For the name of this, why bridal gateway instead of bridal
> API? I guess because it's a gateway to multiple projects. I don't know, I'm totally fine with
> either.
>
> Yeah, I don't know what is involved with the cross machine token for me exactly, or, or like
> where we're at with that. Are we waiting on decisions or just work just hasn't been planned yet?
> But it's needed, right? I'm just gonna run this on one machine, well again once, and I just want
> to be able to manage everything from there. That's part of the attraction of this.

Decided:

1. **Login from v1**: username and password (the human uses 1Password; logging in each time is fine).
2. **The gateway acts with the human's token**; nothing records that an action came through it.
3. **v1 scope: the human's to-dos and decisions**, the things to follow up on, so the
   orchestrator stops relaying them into a scrolling conversation where they get lost.
4. **Retracted items disappear.** When a to-do or decision is resolved some other way, its asker
   retracts it: it leaves the human's list, with an audit trail on the record. What exists
   (advisor, checked): to-dos are tasks, and withdrawing one exists ("To-do withdrawn by ...",
   reason recorded, m-2550). Questions sent as messages to `human` can't be retracted; the
   human inbox (it grows forever, like Gmail) is set aside for now. So v1 shows to-dos and task
   questions; retractable message questions are separate work.
5. **The UI repo is `bridle-ui`**, beside `bridle/` (`/Volumes/Data/work/bridle/bridle-ui`).
6. **Name: `bridle gateway`** (the advisor's suggestion; the human is fine with either):
   "bridle API" already names the daemons' `/v1` API and the `bridle-api` crate, and the
   gateway fronts many daemons.
7. **One gateway, on one machine, manages everything** from there. That needs 3ehu part 1
   (br-8b98, planned, size S: the human's token for another machine's daemons), which waits on no
   decision. Checked: dalek's bridle daemon already listens on its Tailscale address and the NUC's
   daemons answer over Tailscale (k7mw built). A project on a random port, e.g. track-web, needs a
   `[projects]` entry with a fixed port to be reachable from another machine.
