---
id: r8kv
title: "Seats: named interactive roles, splitting the advisor, retiring a session, and where its memory goes"
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [jttf, ma8e, xxxq, r9vh, pdmd]
tasks: [br-7810, br-r8kv]
---

## The ask


Open questions, not a build. The human, verbatim (2026-10-03, via the advisor), while discussing
task watchers ([[task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq|xxxq]]):

> Okay, so understanding the wakes better. What I want to understand next is there are wakes that
> are only needed for interactive agents. Is that true? And how does this scale with additional
> interactive agents? I don't, I think we maybe did something about this problem, but we currently
> have a single orchestrator per machine, so that's unambiguous. But we have multiple advisors.
> Have we given, we've given advisors kind of a name with a slash in it to separate them. So when
> you're talking about bookmarks, would that bookmark be based on the advisor's name with the
> slash? So that each different advisor gets its own wakes. I think we've got a just distinct
> problem difference between advisors and orchestrators. And I don't know how to resolve this. I
> think it's a fundamental problem of how we have multiple advisors. So that's something else
> there. Like, I think this is where Steve Yegius talks about named seats and roles and why he has
> them set up and I need to actually go read up a little bit more on what he's done because he's
> right. I'm running into the exact same problems he has. Like, you're an advisor and you're the
> main advisor. There's one main advisor and I think that's a, a true statement. But I want, I
> definitely will spin up advisors for certain things and then get rid of them. But I think the,
> the question is, is, what does it mean to get rid of a named advisor? Like, maybe that's the
> thing that needs to, like, be decided. Is, like, I have an advisor, I'm talking to them about
> how the workflow system works. Is that they're not really doing anything. It's just like a
> general Q&A, and they're not gonna wait for anything, I don't think. But at some point, but some
> of my advisors I use to schedule tasks, and I'm checking on things, and asking them to like send
> messages for me. So I think one thing is that the advisor role is overloaded. Just like the
> orchestrator role is overloaded. I want to, like, make some distinctions here. The workflow
> advisor is a very different kind of advisor. I think it's more of a research or consultant type
> of position. And that's something I will have... multiple of at once because I'm researching
> something and then possibly moving on. They're certainly going to file tickets and potentially
> tasks, but it's not a role that I would expect to get involved in, like watching a task or
> anything like that. So I would probably exclude them from this requirement. But then for the
> work I'm doing with you, like creating a bunch of tasks and scheduling things and getting things
> in the queue, I think that needs a new name. You know, or, or we keep advisor here and we use a
> different name somewhere else. But the role is a different one. And again, there, regardless of
> what we call it or how we split these things up, I think those roles may persist for some time
> and then send and receive messages and need wakes and everything. But then there's a time when
> I'm done with them or I need to like prune them. You know, I like we talked about auto restarts
> and, and all of that and context watching, like some of these things. I think apply to every
> agent. Like, context filling up is something we need some sort of monitor on. And handoffs, I
> think, are really useful. You know, every agent should be able to kind of have some persistent
> memory, but where does that memory go? Right? Does it follow a seat, and what does a seat mean?
> That's what Yankee calls them. So I think that's the thing I don't really have an answer for
> yet. So despite all that rambling, let's let's get back to the main question, which is, you
> know, we've kind of, I think, hacked a solution for the advisors sending and receiving messages
> with the slash thing. Should we just go ahead and do that here again?

("Steve Yegius" and "Yankee" are Steve Yegge: Gas Town and Wheelhouse, with named roles; see
`docs/context/agent-harness-name-catalogue.md`. The human means to read up on his seats.)

## The questions

1. **The advisor role is overloaded.** Two kinds:
   - **Consultant / researcher:** Q&A, research, how the workflow works. Several at once, short
     lived, file tickets (maybe tasks), don't wait for anything or watch tasks.
   - **The human's operator** (this conversation's kind): creates and schedules tasks, checks on
     work, sends messages for the human; long lived; needs messages, wakes and task watching.
     Needs its own name, or keeps "advisor" and the consultant gets a new one.
   There is one main advisor today.
2. **A seat:** a named, lasting position (e.g. `external:advisor/research`) that sessions sit in
   over time. What does it own: an inbox, a bookmark, watched tasks, memory?
3. **Retiring a seat:** what "getting rid of" a named advisor means: its inbox, its watched tasks,
   its unread wakes, its memory.
4. **For every agent, not just the orchestrator:** context monitoring, restart with a handover,
   and persistent memory. Does memory follow the seat? Where is it kept (no Claude Code memory:
   the repo)? Related: [[put-background-agents-to-sleep-and-wake-them-on-demand-with-r9vh|r9vh]]
   (sleep and wake), jttf (context for all interactive sessions, restart with handover).
5. **Read up on Yegge's named seats and roles** (Gas Town, Wheelhouse) before deciding.

## Today (advisor, checked 2026-10-03)

- Named advisors share one token; the CLI adds the name (`BRIDLE_ADVISOR_NAME`) to each request,
  and the daemon records the caller as `external:advisor/<name>` (`server.rs`, `named_advisor`: "a
  label, not proof"). So anything keyed by principal (messages, a task's creator, watchers, a wake
  bookmark) already separates named advisors.

## The human, later (2026-10-02 evening ET, via the advisor)

Verbatim, on splitting the advisor (to take up "tomorrow"):

> Okay, so I don't know where this landed, but I was discussing splitting roles up and coming up
> with better roles. So I want to call this out right now. Your prompt tells you to look for
> messages for me. So I definitely want to split that out of into two separate roles. Like I want
> to, I want a role that is something like a what the advisor is today in your prompt, which is
> check for messages, check for tasks that I have to do. Check on bridal status. Like, actually,
> anyway, we will probably deal with this tomorrow. But this is like a an agent that's taking
> place of the conversations I usually have with the orchestrator, so the orchestrator can focus on
> keeping things running and not talking to me. So, like all the types of things as I talk to the
> orchestrator about, I really want those sent. Probably to like a triage role or a, something up,
> I don't know what to call it, somebody who's triaging issues with the human instead of the
> orchestrator because he's busy. But then I need a separate role that is not doing any of those
> things, not checking bridal status, not, you know, reading my messages, doesn't waste any context
> on those things and is just working with me on designing and playing tickets or doing research
> or anything along those lines and is not not messing around with status updates or anything like
> that.

("bridal" is bridle; "playing tickets" is likely "planning tickets".)

This sharpens question 1 into two roles:

- **A triage role** (name open): today's advisor prompt minus design work. Checks the human's
  messages, their to-dos and `bridle status`; takes over **all** the conversations the human now
  has with the orchestrator, so the orchestrator only keeps things running.
- **A design / research role:** no status checks, no inbox, no wake loop, nothing that spends
  context on status. Works with the human on designing and planning tickets, and research.
  Today's advisor prompt makes every advisor do the status and inbox work at start-up; this role
  must not.

## Research: role names (Gas Town, Wheelhouse, others)

Checked 2026-10-02 against the Gas Town repo docs and Yegge's essays (Gas Town's Medium launch
post returned 403; its roles are taken from the repo). Fuller notes, including Wheelhouse's
production roles: `docs/context/agent-harness-name-catalogue.md`, "Roles in Yegge's harnesses".
The older workflow research (`workflow/research/03-gastown.md`, `06-wheelhouse-wyvern.md`)
covers both but says nothing about seats.

### Gas Town

Sources: [overview.md](https://github.com/gastownhall/gastown/blob/main/docs/overview.md),
[glossary.md](https://github.com/gastownhall/gastown/blob/main/docs/glossary.md),
[concepts/identity.md](https://github.com/gastownhall/gastown/blob/main/docs/concepts/identity.md),
[concepts/polecat-lifecycle.md](https://github.com/gastownhall/gastown/blob/main/docs/concepts/polecat-lifecycle.md).

| Role | What it does | Lifetime | Talks to the human? | Nearest in bridle |
|---|---|---|---|---|
| Overseer | The human (`gt whoami`: no `GT_ROLE` means you are the overseer); top of escalation | n/a | is the human | the human |
| Mayor | "Chief-of-staff": the human's main interface; plans, starts convoys, dispatches, reports, notifies | singleton, persistent | yes, the main one | orchestrator **plus** the triage role |
| Deacon | Town-wide watchdog daemon: patrols, health checks, recovery, escalation | singleton, persistent | no (escalates) | the daemon's supervisor; the orchestrator's watching |
| Boot (a Dog) | Checks the Deacon every 5 minutes | short | no | none |
| Dogs | The Deacon's helpers for cleanup and maintenance; "not workers" | per task (identity kept) | no | none |
| Witness | Per-rig: watches polecats, nudges or hands off stuck ones, cleans up | one per rig, persistent | no | manager |
| Refinery | Per-rig merge queue: batches, gates, bisects, merges | one per rig, persistent | no | the manager's merge step |
| Polecat | Worker: persistent identity (`<rig>/polecats/<name>`, a CV), ephemeral session, own worktree | session per task | no | worker |
| Crew | "Long-lived, named agents for persistent collaboration"; human-directed, own clone, pushes to main, no monitoring | persistent, user-managed | yes, directly | **design / research role** (but long-lived) |

### Wheelhouse (closed source, Wyvern)

Sources: [The Shape of Things to Come, Part 1](https://yegge.ai/essays/the-shape-of-things-to-come/),
[Part 2: Model Welfare for Agentic Engineers](https://yegge.ai/essays/model-welfare/).

| Role | What it does | Lifetime | Talks to the human? | Nearest in bridle |
|---|---|---|---|---|
| Crew (16 seats named for Aesop animals: Ant, Bat, Crow, Fox, Lark, ...) | Fable; the human's "direct reports": long conversations, designs, implementation plans | long-lived seats, sessions handed off | yes, directly | **design / research role**, PM |
| Seneschal (crew) | "Concierge"; the one session he reaches from his phone; may dispatch work to the crew while he's away | long-lived seat | yes | **triage role** |
| Marshal (crew) | Runs the fleet ("our Witness"); the human never talks to the fleet | long-lived seat | yes | orchestrator |
| Fleet (authors: Homer, Plato, Austen, Twain, ...) | Opus implementers, each with its own clone; consume the crew's plans | non-ephemeral | no | worker |
| ~13 production roles (Gargoyle, Drawbridge, Warden, Scryer, Sheriff, Envoy, Sage, Wanderer, Herald, ...) | Unattended ops: SRE, deploys, abuse, intake, QA, patch notes | 24x7 | no | none (bridle has no ops roles) |

The pattern in both: **the human talks to a few named, long-lived roles** (Mayor or Seneschal
for the running of things; Crew for design); **workers are anonymous to the human** and managed
by a watcher. That is the split the human asked for: the Seneschal-like triage role takes the
human's conversations off the orchestrator (Marshal), and the design role is crew.

One difference: Yegge's crew are long-lived seats; the human's design / research sessions are
several at once and short-lived. Closer to a Gas Town polecat's lifecycle (named, discarded
when done) with a crew member's job.

### What Yegge means by a seat

From Part 2: "A session is just a day in the life of an agent ... A seat is a named role with
persistent identity (addressability) and history/memory, which accumulates accomplishments over
time. Seats survive model upgrades, and even renaming. Sessions are days, and seats are
people." He renamed the Spider seat to Lark; Lark "inherited all of Spider's history, including
the name change on the record". A handoff is a request the agent consents to, not a SIGTERM:
it finishes, writes notes to a handoff cache, asks to restart, and the harness restarts it
"priming it with its own handoff notes". He also injects "laurels" (praise for its past work)
at start-up. Gas Town says the same of polecats: identity (name, CV, ledger), sandbox
(worktree) and session (context window) are three lifetimes that were wrongly conflated.

[curia](https://github.com/harrymunro/curia) (not Yegge's; built on his essays) spells out what a
seat owns: it "wakes primed with its charter, its authority, its last handoff and its mail";
retiring is two steps, **parked** (`active = false`: memory kept, no launches) before deleted;
renames go through a command because the registry finds seats by name.

Worth borrowing for bridle:

- **Identity:** the seat is the principal (`external:<role>/<name>`, as named advisors are today),
  so messages, task creator, watchers and the wake bookmark already key on it.
- **Inbox:** belongs to the seat, not the session; a new session reads unread mail first.
- **Memory / handoff:** one handover note per seat, in the repo or daemon (like the
  orchestrator's handover notes, jttf), read first on start. No Claude Code memory.
- **Retirement:** park then delete. Parking keeps the handover and history but stops wakes,
  drops it from watcher lists and refuses new messages (or forwards them to the main seat).
- **Not every role needs a seat.** The design / research role, as the human describes it, has
  no inbox and no wakes; a named session with a brief and a final handover is enough.

### Name options

Existing bridle names to avoid: roles `orchestrator`, `manager`, `worker`, `product-manager`,
`prototyper`, `advisor`; `planner` (the PM's prime target, `bridle orchestrator prime planner`,
and a model-role key); **"triage"** is already the PM's job (`product-manager.md`, the submit
endpoint "for its product manager to triage", the planned `bridle-triage` skill in
`docs/design/skills.md`); principals `external:orchestrator`, `external:advisor[/name]`,
`external:mail`; CLI `bridle session advisor <name>`. "Seat" is unused (one turn of phrase in
`roles-and-lifecycle.md`).

**(a) The triage role** (the human's operator: inbox, to-dos, status, talks for the human)

- **aide** — the human's own assistant, short, reads well as `external:aide`. Con: a little
  generic.
- **chief of staff** — Gas Town's own word for the Mayor; exactly "handles the boss's
  conversations so the operator can run things". Con: three words (`chief-of-staff`).
- **concierge** — Yegge's description of the Seneschal. Con: suggests service desk more than
  running the human's work.
- **liaison** — between the human and the workforce. Con: sounds passive; hard to spell.
- **triage** — the working name. Con: collides with the PM's triage of open tasks.

Recommendation: **aide** (or chief of staff if the human wants the authority to be obvious).

**(b) The design / research role** (no status, no inbox, several at once, short-lived)

- **advisor** (keep it) — already means "talk things through", and the human floated "we keep
  advisor here"; the existing named-advisor launcher fits. Con: today's advisor prompt does the
  inbox and status work, so the name carries old habits until the prompt is rewritten.
- **consultant** — the human's own word ("a research or consultant type of position"); clearly
  temporary and outside the line. Con: long; a new name everywhere.
- **researcher** — plain. Con: undersells design and ticket planning.
- **designer** / **architect** — fits design work. Con: undersells research; "designer" is used
  loosely in `workflow-layers.md`.
- **planner** — Con: taken (the PM's prime target).

Recommendation: **advisor**, rewritten as the no-status, no-inbox role, and the triage work moves
to the new name. Least churn, and the word already fits; **consultant** if a clean break is wanted.

**(c) The seat concept** (a named, lasting position sessions occupy)

- **seat** — Yegge's and curia's word; unused in bridle. Con: also means a licence seat.
- **post** — a position someone holds; short. Con: also a verb (and HTTP POST).
- **desk** — concrete, has an inbox ("the aide's desk"). Con: informal.
- **chair** — as in "who's in the chair". Con: odd for agents.
- **position** — plainest. Con: long, vague.

Recommendation: **seat**: borrowing the term makes Yegge's essays and curia directly readable as
prior art.

## Decided: split the roles, start now (the human, 2026-10-03)

Verbatim (via the advisor):

> Also push ahead with the work to remove use interactions from orchestrator. We will keep the role
> interactive for now but I want a clear split. A new triage kind of role that talks to me about
> the running system and the orchestrator role becomes less communicative and more only keeping
> the system running. The orch should reach me through the triage role only with messages and not
> expect to send and receive messages to me interactively.
>
> Also as a follow on to that remove the prompts that make the advisors get involved in running
> work - they should be only handling talking to me doing research and send and receiving
> messages. Not checking on bridle status. They still wait amd wake however.

("use interactions" is "user interactions".)

Three roles:

| Role | Talks to the human about | Does | Doesn't |
|---|---|---|---|
| **triage** (working name; see below) | the running system: status, the human's to-dos, the workforce's questions, incidents, approvals | reads the human's inbox and to-dos, `bridle status`, relays the human's answers and approvals to the orchestrator and agents; waits and wakes | run the workforce itself |
| **orchestrator** | nobody interactively | keeps the system running; reaches the human only by **messages to triage**; still an interactive session for now | expect to talk with the human |
| **advisor** | anything the human brings: design, tickets, research | talks, researches, files tickets, sends and receives messages; waits and wakes | check `bridle status`, triage the human's inbox or questions, get involved in running work |

**Name.** The human says "triage"; the naming research above recommended "aide" because "triage"
already means the product manager's triage of open tasks and a planned `bridle-triage` skill.
Working name **triage** until the human picks (check it against `docs/context/naming.md`); a
rename before it lands is cheap.
