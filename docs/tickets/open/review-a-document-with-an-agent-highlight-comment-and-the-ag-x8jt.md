---
id: x8jt
title: "Review a document with an agent: highlight, comment, and the agent replies or revises"
kind: feature
opened: 2026-10-02
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [essy, v8kn, hvxk, 6yb4, k4wq, yyzm, yj38, r9vh]
tasks: [br-rp53, br-aj9d, br-5paw, ui-acf0, br-qttb, ui-c39e]
---

## The ask


The human, verbatim (2026-10-02, via the advisor):

> Okay, great. So what I want to talk about is how we, we've recently started some work on a
> bridal gateway and a proposal for a bridal UI that's a separate project that's going to be
> written in TypeScript. And I want to talk about how I'd like that to work for authoring and
> reviewing tickets and design questions. So things work pretty well with an agent where I can
> have a chat and read things, but there's a real limit to the way a chat interface works. Chat
> scrolls up, the agent responds, or asks some questions, it's kind of a pain to scroll up and
> then type responses at the same time in Claude Code or in any interface, really. And it's kind
> of just a frustrating experience for me in a lot of cases. So what I want out of a UI is... And
> I think this works with tickets mostly. I'm not sure if it would apply anywhere else, but other
> kinds of conversations and decisions, like designs and plans possibly, we haven't really
> figured out how using kind of an open spec-like process would work here. I don't think we've
> figured it out. But I'm going to kind of narrate some of my thoughts, and this is likely a new
> ticket to open. And but please search around for existing tickets that cover this. So, what's
> going on is I have a bunch of tickets, a bunch of tickets I haven't looked at at all. I can
> probably just throw away or ignore. That's fine. But also, there's tickets and discussions that
> we have around design, like the design for the. UI. It's E-S-S-Y, I believe. And there was a
> lot of back and forth on this, and it actually went pretty well because I had some time to
> just focus and look at it. But I still had to do this whole scrolling around, trying to
> remember the numbers and answering the questions. What I think is more effective is I want to
> be able to open a document And then I want to be able to highlight a section of the document
> and then add comments on it. Basically similar to what you get in Google Docs. I want to review
> it, highlight something, add some comments, and then pass it back to the agent. The comments,
> they might go immediately or I might bash them up. That's something to maybe think about. Or
> the agent may respond to them immediately or may wait. Probably just immediately at the first
> is easiest. So I would look through the doc, I would highlight a section and say, what does
> this mean, please clarify. And the agent can then either reply to my comment in a thread, or
> they can just you know rewrite the section. So if I say like, rewrite this, make it more clear,
> then the agent would just do that and then the comment would be resolved because they've done
> it. Or if I say like, I don't understand this, please give me more detail. Then the agent could
> reply to my comment further, or even do both, whatever the, you know, it's, it's up to the
> agent and what I've asked for. I might say, like, expand this section, and then they would do
> that. You know, think like how Google Docs work, or, or a little bit like how comments on a PR
> work in GitHub. That's kind of what I'm thinking about. And this concept is something I want to
> experiment with and then codify as a repeatable thing within Bridal. And what I mean by that is
> this sort of presenting something to the user, to the human, and then getting a bunch of
> responses is, is a really key and core workflow. This is particularly the case with like
> whether you're building a UI or building these documents or trying to understand an
> architecture. You know, I think at some point I want to go back to some previous work that I
> did under the PyHarness project where I was kind of working with interactively building a
> presentation along with an agent. But, you know, beyond that, I want an interactive surface for
> the agent and the human to draw diagrams and then adjust and comment have a conversation about
> them.
>
> The visualization part of this would, would come later, but it's something to consider. So
> again, it would have a similar like kind of comment flow. And I think I did, I did some of this
> in the PyHarness project. I also played around with this in a different project that used
> Draw.io for it, but I don't think that came out great. Um, you know, Draw.io is a pretty nice
> interface in general, but I, I think building something custom is, is probably fine. I don't
> think we need fantastically complex diagramming tools. Um, but yeah, I, I want to be able to
> like have the agent draw a diagram. I can add some comments and questions to it, or I can
> change the diagram, and then that gets can get sent back to the agent, uh, showing what I've
> changed, like recording. I think it sends a diff of the diagram, um, so. that the agent
> understands the changes that I've made and can, of course, see the new diagram.

("bridal" is "bridle"; "bash them up" is likely "batch them up". "PyHarness" is **pi/harness**,
`/Volumes/Data/work/pi/harness`: the human's interactive web-based harness for experimenting with
the Pi coding harness and other models, e.g. Kimi 2.7; the human, 2026-10-02.)

## What it asks for

1. **Review a document by commenting on it, not by chatting.** Open a document (a ticket, a
   design doc, a plan) in the UI, highlight a passage, and comment on it, as in Google Docs or a
   GitHub PR review. Chat makes the human scroll back, remember numbers and answer at the same
   time; the essy design rounds went well but needed exactly that.
2. **The agent answers each comment as it sees fit:** reply in the comment's thread, revise the
   passage (and resolve the comment, since it's done), or both, depending on what the human
   asked ("what does this mean?", "rewrite this more clearly", "expand this section").
3. **Timing:** comments go to the agent at once for a start. Batching a review (send several at
   once) and the agent waiting before it answers are to think about later.
4. **Experiment first, then codify it in bridle** as a repeatable workflow: presenting something
   to the human and taking their responses is a core loop, for documents, UIs and understanding
   an architecture.
5. **Diagrams are split out:** the same loop for diagrams is
   [[draw-and-edit-diagrams-with-an-agent-comments-human-edits-se-yyzm|yyzm]] (the human,
   2026-10-02).

## How it fits (advisor, checked 2026-10-02)

- Related tickets (none yet says how comments on a document work):
  - [[a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]]: the human web
    UI (bridle gateway plus `bridle-ui` in TypeScript). Its v1 is to-dos and decisions; this
    would be a later screen there.
  - [[everything-readable-and-editable-through-the-daemons-file-ba-v8kn|v8kn]]: closely related.
    The UI is a separate program, so it can read and write tickets and docs (files in a clone)
    only through the daemons' API; v8kn adds that, and lists "ticket edits and comments" as its
    second step. A UI for this ticket needs v8kn's reads and writes. v8kn doesn't say what a
    comment is; this ticket does.
  - [[refining-a-task-with-the-human-before-it-ships-hvxk|hvxk]]: refining a proposal with an
    agent before it ships (an OpenSpec-like loop). This review surface is a likely way to do it.
  - [[a-prototyper-role-in-the-base-workflow-build-only-from-the-p-6yb4|6yb4]] (prototyper) and
    [[a-human-surface-beyond-the-cli-k4wq|k4wq]] (a human surface beyond the CLI).

## Comments live in the document, as plain text (the human, 2026-10-02)

The human, verbatim (via the advisor):

> Yeah, I think maybe review how comments are stored along with a task. You know, I, th I think
> we want to KISS, keep this as simple as possible. Um, my first thought would be, you know, use
> something like uh, how MIME types are attached to an email. Just uh, throw a couple dashes in
> there or some other kind of separator and then put the comment information in the doc right
> along with it. Um, maybe track it, track it by line number or uh, I don't know, maybe even just
> you know, the comment follows the standard markdown syntax. That, that might be best, really.
> Um, you know, after a passage, um, uh, you know, after the line that I've highlighted, or part
> of a line that I've highlighted, just add a indent there, and maybe repeat, you know, in quotes,
> the part that was highlighted, or the characters or something in quotation marks and then uh,
> put my comment there say you know from human or whatever and then just thread it right in place
> I'm not sure uh, but I definitely want to keep it simple I definitely want to keep everything in
> that document um, I, I think I love plain text I think it's a great way to do it and that just
> means that if I browse it by myself I can read it no problem and if we Uh, when we build the UI,
> the UI just needs to be able to detect that those are comments that float off to the side, and
> again, no problem. Um, comments, you know, they get resolved, possibly just go away completely,
> or um, could be tracked maybe at the bottom of the document saying, you know, the human left a
> comment on this section and uh, that section was rewritten so the comments resolved something
> along those lines maybe it would work um, but I would I would also like definitely be supportive
> of throwing away resolved comments after a turn or two or just having a oh maybe a command that
> just uh, cleans them up at a later point in time you know we're using Git to version these
> documents anyway, and changes should be committed. Um, so whenever, anytime we delete like
> something like that, a resolved comment, it's always discoverable in the Git history.

The direction (not yet a format):

- **Keep it simple; everything stays in the document, in plain text** that reads fine without
  the UI. The UI only has to recognize comments and show them to the side.
- **A comment sits right after the line it's on**, quotes the highlighted text, says who wrote
  it, and its replies thread in place below it. Plain markdown, no separate store and no line
  numbers (they drift when the agent rewrites).
- **Resolved comments go away**: deleted at once, after a turn or two, or by a cleanup command;
  maybe a short line at the bottom noting what was resolved. Documents are committed, so git
  history keeps every deleted comment.

Advisor's sketch of a format, to try: an Obsidian-style callout, which is plain markdown, reads
fine raw, and renders as a box in Obsidian (the human uses Obsidian for notes):

```markdown
The gateway acts with the human's token; nothing records that an action came through it.

> [!comment] human, 2026-10-02, on "nothing records"
> Why not? Wouldn't an audit want it?
>
> **advisor:** The task's history already names the principal (the human); the route it
> came by doesn't change who acted. Rewrote the line to say so.
```

A side effect worth using: **the first experiment needs no UI.** The human can type comments in
this form in any editor, and an agent can answer them in the file. The UI (essy, through v8kn)
then becomes a nicer way to write and read the same text.

## Which agent answers (the human, 2026-10-02)

The human, verbatim (via the advisor):

> Which agent answers is a great question. I hadn't thought about that one. I don't think a brand
> new agent for every comment is appropriate at all. So maybe, I don't know. Let's give me some
> ideas here. The, I was thinking that, you know, it's, this is kind of the advisor's job. So I
> don't know if that means literally the, an interactive advisor, or maybe we spin up an advisor
> agent when I'm working on a ticket. I feel like that might, that might be good. Like we have, if
> Brewin Bridal sees that comments are coming in on a document, it would create an advisor type. I
> mean, it would probably be a different role a little bit, you know, maybe it's a, maybe it's a
> unique role here, but similar to advisor, not dealing with current issues or anything, but like
> a, a specific agent that would spin up that would be related just to this document. That we're
> working on, and it And that agent would be assigned to questions about that ticket. And it, it
> must be able to like read other tickets and do the normal things that an advisor would do. I
> think it, it would have general permissions over certainly all the tickets and tasks and things
> to be able to inspect the state of the system and be able to like send messages and create new
> tickets, possibly create tasks as well. You know, I'm, I'm sure that I would make a highlight and
> say, you know, create a new ticket for this part of the document and then delete it. That seems
> really likely. I'd have to think through what else I might want this to do. But this, that agent
> handling something like this is going to be another repeated things that comes up. Not an
> interactive agent, but a specific kind of agent to respond to something.

The human's direction:

- **Not a new agent per comment.** One agent per document under review, of a role like the
  advisor's but focused on that document, not on current issues. Bridle starts it when comments
  arrive on the document.
- **Advisor-like powers:** read every ticket and task, inspect the system's state, send
  messages, file tickets, maybe create tasks. A likely ask: "make a ticket from this part, then
  delete it here".
- **It's one case of a general pattern**, an agent of a specific kind that responds to
  something: [[responder-agents-an-agent-for-each-kind-of-incoming-item-wit-yj38|yj38]].

Options (advisor), with a recommendation:

- **A. The interactive advisor answers.** Nothing new to build, but it competes with the chat
  and carries every other topic in its context.
- **B. One agent per document, kept for the review (recommended).** Started on the first
  comment, it stops when idle and is resumed (same session, so it remembers earlier rounds) when
  the next comment arrives; closed when the review ends or the ticket resolves. Its record goes
  on the ticket's task thread. Clean context per document fits hvxk.
- **C. One standing responder per project** for every document. Fewer agents, but documents
  share and crowd one context.

**Decided (the human, 2026-10-02): B.** The human, verbatim (via the advisor):

> I definitely like B in general. I think it's the right approach. Probably the we need a kind of
> an expiry for this agent.

> I think number A is kind of a nifty option. You know, maybe I'm having an interactive chat and
> I want to add some comments just for the chat and take it back to chat, but it kind of seems
> like just mixing two different things up too much. So I think my preference is with B.

- **B: one agent per document**, with an **expiry**, and a **governor** so commenting on many
  documents at once doesn't start many agents. Putting agents to sleep and waking them, the
  expiry and the cap are [[put-background-agents-to-sleep-and-wake-them-on-demand-with-r9vh|r9vh]].
- A (the interactive advisor) stays a possible extra for comments made during a chat, not the
  design.

- Still open: how a revision shows the human what changed (git diff of the commit, or the UI
  marks changed passages), and batching comments into one review round.

## Example: one review round (advisor, 2026-10-02)

The human asked to see what it looks like (they were on a plane, no trial possible). A passage
from essy's design, after the human commented twice and the document agent answered:

```markdown
- **Login:** username and password, `argon2` hash in machine config, an `HttpOnly`,
  `SameSite=Strict` session cookie.

> [!comment] human, 2026-10-02 14:05, on "`SameSite=Strict`"
> What does this mean? Plain words please.
>
> **docs agent, 14:06:** The browser sends the login cookie only to pages the gateway itself
> served, never to requests started by another site, so a malicious page can't act as you.
> Want that sentence added to the doc?
>
> **human, 14:20:** yes

- **Reached over Tailscale**, encrypted; nothing is exposed to the internet.

> [!comment] human, 2026-10-02 14:07, on "Reached over Tailscale"
> Make a ticket for serving this over HTTPS on the internet later, then delete this line.
>
> **docs agent, 14:08:** Filed ticket abcd ("serve the gateway over HTTPS"). I'll delete the line
> once you resolve this.
```

After the human's "yes", the agent adds the sentence, deletes the first thread, and commits with
the comments in the message; a resolved note goes at the bottom of the document:

```markdown
---
Resolved comments (deleted, see git history):
- 2026-10-02, human on "SameSite=Strict": explained; sentence added under Login.
```

What the example shows:

- The raw file reads fine in any editor; in Obsidian each `[!comment]` is a box.
- Replies are bold names inside the quote: no nesting to track.
- The quoted words anchor the comment; if a rewrite removes them, the comment is still right
  after its passage, and the agent resolves it in the same commit.
- A thread can end in an action (a ticket filed, a line deleted) as well as an answer.

## Format approved; batching; what changed; tags (the human, 2026-10-03)

The human, verbatim (via advisor doc-review, ~8:45 PM ET):

> Approve the comment format.
> Debounce and batch comments over like 5-10 minutes.
>
> Is 2 about showing what's new? Hrm. I can't think of a good solution in text. We could use the
> git diff if/when needed. Defer the functionality. When the agent changes the doc in response to
> a comment, it should add a follow up comment and tag me.
>
> Tags in comments should be then marked read when I read them or the agent.
>
> I want to get this moving quickly.

Decided:

- **The format is approved**: the `> [!comment] <who>, <when>, on "<quoted words>"` callout above,
  replies as bold names inside it, resolved threads deleted with a note at the bottom.
- **Comments are debounced and batched**: the agent gets them as one round once the human has
  stopped commenting for about 5-10 minutes.
- **Showing what changed is deferred**; the git diff serves when it's needed. Instead, **when the
  agent changes the document for a comment, it adds a follow-up reply in that thread tagging the
  human**.
- **Tags are marked read** when the tagged party (the human or the agent) reads them.

## Tag format; approved to build in three slices (the human, 2026-10-03)

The human, verbatim (via advisor doc-review, ~9:00 PM ET):

> Approve all three to build. Approve tag format

> The ticket to test is gtzx

- **Tag format:** `@human` or `@docs-agent` at the start of a reply (`**docs agent, 14:08:** @human
  Rewrote the Login bullet; see the commit.`). Read is marked by appending `(read)`:
  `@human (read)`. The agent marks its own tags when it takes a round; the UI marks the human's
  when they open the thread; until the UI, a reply in the thread counts as read.
- **First trial document:** gtzx. Until slice 1 lands, advisor doc-review acts as its document
  agent (the human comments in an editor and says "go").

Approved slices, in order, each usable alone:

1. **Document-reviewer role** (bridle): a role prompt with the approved format and how to reply,
   revise (follow-up reply tagging `@human`), mark tags read, resolve (delete the thread, note at
   the bottom) and commit each round. One agent per document, started by hand by the orchestrator
   and told "go".
2. **Bridle notices comments** (bridle): watch the documents under review; when new comments have
   been quiet for 5-10 minutes, start or resume that document's agent with the batch. Uses the
   existing agent stop/resume and a simple cap on how many run at once; doesn't wait on the rest of
   r9vh. The expiry per document agent comes with it.
3. **Document view in the web UI** (bridle-gateway + bridle-ui): open a document, comments to the
   side, highlight to add a comment, tags marked read on opening the thread. Needs a narrow piece
   of v8kn: the gateway reads and writes one document file and commits it.

## Filed (orchestrator, 2026-10-04)

Slice 1 br-rp53, slice 2 br-aj9d. Slice 3 is split by repo: br-5paw (gateway reads, writes and
commits one document file) and ui-acf0 in bridle-ui (the document view, built on br-5paw). The old
br-2ec0 was dropped.

## Review now, from the CLI and the UI; sent comments are marked (the human, 2026-10-04)

The human, verbatim (via advisor doc-review):

> Add a bridle command to perform the review on a document immediately and a button in the UI
> also to request the review.
>
> Comments that have been sent for review should be marked as such and not resent if the button
> is pressed again, unless requested.

- **Review now:** a `bridle` command and a UI button send a document's pending comments to its
  agent at once, skipping slice 2's quiet period.
- **Sent comments are marked** as sent, and pressing the button (or running the command) again
  doesn't resend them, unless the human asks for a resend.

The human, verbatim (2026-10-04):

> Comment threads probably need a UID as well but we could wait on that

- **Thread IDs: deferred.** Threads have no ID yet; they're found by position and quoted words.
  An ID would key "sent" and "read" state and UI links reliably once wanted.

**Decided (the human, 2026-10-04): A, the sent mark lives in the file.** The human, verbatim: "A".

- When bridle sends a batch (after the quiet period, or on review now), it appends `· sent YYYY-MM-DD HH:MM`
  to the line of each thread's newest human entry: the `[!comment]` header, or the human's latest
  reply line. Plain text, visible in any editor, and a restart doesn't resend.
- Review now and the quiet-period send both skip entries already marked sent; `--resend` (and a
  resend option in the UI) sends them again.
- To build: `bridle review now <path> [--resend]` and the mark (bridle: daemon `doc_watch.rs`,
  CLI, a gateway route the UI button calls, docs/design daemon.md and cli.md), and the button in
  bridle-ui's document view.

Review now and sent marks: br-qttb (bridle: command, sent marks, gateway route) and ui-c39e
(bridle-ui: the button, after br-qttb lands).

The human, verbatim (2026-10-04): "Let's do YYYY-mm-Dr HH:MM for sent" ("Dr" read as DD): the
mark is `· sent 2026-10-04 21:14`.
