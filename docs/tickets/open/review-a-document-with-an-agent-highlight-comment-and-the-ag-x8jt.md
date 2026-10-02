---
id: x8jt
title: "Review a document with an agent: highlight, comment, and the agent replies or revises"
kind: feature
opened: 2026-10-02
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [essy, v8kn, hvxk, 6yb4, k4wq]
tasks: [br-2ec0]
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

("bridal" is "bridle"; "bash them up" is likely "batch them up".)

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
5. **Later: the same loop for diagrams.** An interactive surface where the agent draws a
   diagram and the human comments on it or edits it; the human's edits go back to the agent as a
   diff plus the new diagram. Simple and custom is fine; Draw.io was tried elsewhere and didn't
   come out well. Prior work to look at: interactively building a presentation with an agent in
   the PyHarness project.

## How it fits (advisor, checked 2026-10-02)

- Nothing on file covers inline, anchored review. Nearest:
  - [[a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]]: the human web
    UI (bridle gateway plus `bridle-ui` in TypeScript). Its v1 is to-dos and decisions; this
    would be a later screen there.
  - [[everything-readable-and-editable-through-the-daemons-file-ba-v8kn|v8kn]]: tickets and docs
    readable, editable and commentable through the daemons ("read-only first, then ticket edits
    and comments"). This needs its read and write of ticket and doc files, and adds comments
    anchored to a passage, with threads and resolve.
  - [[refining-a-task-with-the-human-before-it-ships-hvxk|hvxk]]: refining a proposal with an
    agent before it ships (an OpenSpec-like loop). This review surface is a likely way to do it.
  - [[a-prototyper-role-in-the-base-workflow-build-only-from-the-p-6yb4|6yb4]] (prototyper) and
    [[a-human-surface-beyond-the-cli-k4wq|k4wq]] (a human surface beyond the CLI).
- Open points for the design, not decided: where comments live (the task thread, beside the
  file, or the daemon's store), how a comment stays anchored when the agent rewrites the passage,
  which agent answers (a fresh one per review, focused on that document, fits hvxk's clean
  context), and how a revision shows the human what changed.
