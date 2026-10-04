# Naming things

How we name what we build and run here: roles, commands, states, kinds, documents, projects.
These are the human's preferences; agents follow them when they propose or choose a name.

## The human's words

Verbatim (2026-10-03, via the advisor):

> I want to create a new document that's about how to name things. This is for naming things that
> we work on here in this system in the agentic era.
>
> One of the important things about naming things, for me, just in general, is that I like my
> names to be pretty short. I like them to be very descriptive, though, and clear, and I don't
> like to avoid ambiguity. These things can all be conflicting, so that can be a problem.
> Generally speaking, I want to keep names that match what an average person would expect, and
> that average person is generally me. That's the idea.
>
> There are a couple additional notes I want to add for agentic names, and one is this: I use
> several dictation systems for talking to my agents, and each one of those is going to come up
> with a different idea of what the word means that I'm using. One particular example is
> "bridle. It works fine when I type it, although it's a little annoying to type. When I say it in
> a message, it's been coming out like "bridal shower" through Claude because the Claude
> transcription doesn't know what I'm talking about.
>
> Now, in Wispr Flow, I can fix a lot of these problems. It has a dictionary option, so when
> something I say doesn't come out right, I can go at it there, and it usually picks the
> override. I think this is important going forward.
>
> The other thing is that this applies to things like our role names, which is really important,
> and system names. Things like "ticket" and "task" are basically synonyms, so I'm pretty
> concerned about the naming as well as the functionality. When two things have the same name,
> that's often an indication that they don't have good distinction in the boundaries between
> them.

("I don't like to avoid ambiguity" means the human wants to avoid ambiguity.)

Then (2026-10-03):

> What I've been talking about so far applies to a couple of things:
>
> 1. System tools, commands, things that we might do with the system: deciding ticket, task,
>    gateway, rules, workflow, events, messages, notifications. That's the kind of thing we need
>    to think about: the proper names of things, as sort of generic names.
> 2. How we name something, like a personal name in a sense, like an assigned name. For example,
>    this computer is called Dalek, so that's a reference to Doctor Who. I picked it years ago. It
>    happens to work fine because it's a word that all of these transcription systems understand.
>    That's really fantastic, but the Intel NUC is called a NUC, and that gets mis-transcribed all
>    the time. Secondly, it's not a name; it's hardware, so it's fine. It indicates to me which
>    machine it is. I also have a Windows PC I might hook up. I could be stupid to call that
>    Windows, just like it'd be stupid to call this thing MBP. I own several MacBook Pros, so I
>    would never choose that as a name for my computer. The Nook kind of just picked up that name
>    because it didn't have another one, so we need to rename the Nook, and that's fine. Generally
>    speaking, I want to be reminded of these conventions and ideas that I have around naming
>    whenever we're coming up with names for something. Orchestrator might define the role
>    accurately, but it might not. It's super long to type and super awkward, so it's actually
>    fine to dictate that. It gets really annoying to type, so I ended up shortening it to ORCH,
>    which is reasonable as long as we all understand what that means.

("the Nook" is the NUC, mis-transcribed: an example of the problem.)

## Two kinds of name

1. **Generic names: what a thing is.** The system's concepts, tools and commands: ticket, task,
   gateway, rule, workflow, event, message, notification, the roles. These must describe the
   thing accurately and be distinct from each other (see "Same name, blurred boundary").
2. **Proper names: what one thing is called.** An assigned, personal name for one particular
   machine, project, agent or seat, like `dalek` (the human's Mac, after Doctor Who). It needn't
   describe anything; it must pick out that one thing and survive dictation.
   - **A type is not a name.** "NUC", "Windows", "MBP" say what kind of hardware it is, not which
     one; the human owns several MacBook Pros, and may add a Windows PC. Name the machine, not
     its kind.
   - **The NUC needs a real name.** It took "NUC" for lack of one, and dictation hears "Nook".
     Renaming it is wanted (not yet done).

## A design rule

The human (2026-10-03, verbatim):

> This isn't a rule at all, I don't think. Actually, let me take that back. I think this is an
> architectural rule or a design rule. It's a design rule that should be part of designing the
> solution or the proposal. Yes, in that sense, this is a design rule. It's not a worker rule, I
> don't think at all, because the names of things should be decided in the design.

So names are settled in the design or proposal, by whoever designs (the orchestrator, advisors,
the project manager, and the design/research role to come), not by managers or workers while
building. A worker who finds a thing without a good name raises it; it doesn't invent one.

## Remind the human

Whenever we're choosing a name (a role, a command, a state, a project, a machine), point the
human at this document and check the candidates against it: say them aloud, look for
sound-alikes and near-synonyms, and say how each fares. The human asked to be reminded every
time.

## Principles

1. **Short.** One word if it can be; two at most for anything said often.
2. **Descriptive and clear.** The name says what the thing is or does.
3. **Unambiguous.** One name, one meaning, within this system and against everyday English.
4. **What an average person expects**, and that person is the human. Plain words over jargon
   or cute names. If the human would have to look it up, pick another.

When these conflict, clarity and no ambiguity beat brevity.

**Typed and spoken differ.** A long name can be fine to say and tiresome to type: "orchestrator"
describes the role (maybe) and dictates well, but is awkward to type, so the human types "orch".
A short form is fine **if everyone knows it**: write it down here, and agents read it as the full
name.

**Short forms are for typing and tight spaces, not for prose to the human.** The human, verbatim
(2026-10-03):

> I wanted to clarify something. You started saying "orch" in your writing back to me. I actually
> don't like that at all. I don't have any problem reading the word "orchestrator." I just have a
> problem typing it. I think it's kind of like a shorthand that I would use with you, or a
> shorthand that we might use when assigning a name, like in a UI.
>
> One of the reasons to add this to that context naming document is to favor short names for
> things or abbreviations. We're going to run up against this: we put things in lists in a user
> interface, and we put things in lists in a document. We might be making tables, naming
> projects, and if you spell out every single word really long, then scrolling horizontally
> always becomes a problem. That's another thing for the naming context stack.

- **Agents write the full name to the human** ("orchestrator", not "orch"). The human may type
  the short form; agents read it as the full name and don't echo it back.
- **Short forms belong where space is tight:** UI lists and columns, table cells, identifiers,
  project and agent names. Long names in lists and tables force horizontal scrolling, so favour a
  short name, or an agreed short form, there.

Agreed short forms:

| Short | Means |
|---|---|
| orch | orchestrator |

## Names get spoken

The human talks to agents through several dictation systems (Claude's own transcription, Wispr
Flow, others). Each guesses differently at an unfamiliar word, so a name is only good if it
survives being said aloud.

- **Say it before choosing it.** Prefer common words that dictation spells one way. Avoid
  homophones and near-homophones: "bridle" comes out as "bridal" (or "bridal shower").
- **Avoid names that sound like another name here**, even if they're spelled differently.
- **Fix what slips through in the dictionary.** Wispr Flow has a custom dictionary that usually
  takes the override; when a name keeps coming out wrong, add it there. Claude's transcription
  has no such fix, so the name itself has to be robust.
- **Agents read through mis-transcriptions.** When a message says "bridal", it means bridle;
  read for the likely word, and quote the human verbatim in records (with a note, as tickets do).

Known mis-transcriptions:

| Said | Comes out as |
|---|---|
| bridle | bridal, bridal shower |
| Yegge | Yeggy, Yegius, Yankee |
| NUC | Nook |
| 3d, 4d (in IDs) | 30, 40 |
| aide (said alone) | aid (Claude's transcription; "my aide" in a sentence comes out right) |

**IDs get spoken too.** Random IDs are the worst case: no dictionary entry can fix them, and
some are ambiguous in English. The human (2026-10-03): "the number 3, the letter D, and then the
number 4 and the letter D ... when I say that out loud, it's 3D, 4D ... it basically sounds
identical to 30, 40." When we design an ID, check how it sounds, or let the human refer to things
by name instead. See [[ids-the-human-can-say-aloud-task-and-ticket-ids-that-survive-nkd9|the
speakable-IDs ticket (nkd9)]].

## Same name, blurred boundary

Two names that mean nearly the same thing usually point at two concepts without a clear line
between them. Treat a pair of near-synonyms as a design question, not just a naming one: define
the boundary first, then name each side so the names make the difference obvious.

Pairs to look at (examples, not decided):

- **ticket / task:** [[tickets-and-tasks-why-both-k7tm|k7tm]].
- **project manager (was product manager) / manager:** "PM" is ambiguous, and both sit beside
  "manager" ([[the-product-manager-role-is-really-a-project-manager-who-hel-7r2c|7r2c]]).
- **advisor / orchestrator, and the roles split out of them:**
  [[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]] (its research
  section has name options from Gas Town and Wheelhouse).
- **message / wake / note / comment:** several words for "something reaches an agent".
- **agent / session / principal / seat:** who is acting, and for how long.

## Related

- [[agent-harness-name-catalogue]]: names other harnesses use for the same things.
- `workflow/base/rules/ticket-references.md`: how to name a ticket to the human (lead with the
  start of the file name, not the bare ID).
