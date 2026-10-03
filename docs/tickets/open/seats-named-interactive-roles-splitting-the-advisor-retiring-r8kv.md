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
tasks: [br-7810]
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
