---
id: yyzm
title: "Draw and edit diagrams with an agent: comments, human edits sent back as a diff"
kind: feature
opened: 2026-10-02
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [x8jt, essy]
tasks: [br-19b4]
---

## The ask


Split from [[review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt|x8jt]] (reviewing
a document by commenting on it) at the human's request (2026-10-02): the same comment loop,
for diagrams. The human, verbatim (2026-10-02, via the advisor; the full message is in x8jt):

> You know, I think at some point I want to go back to some previous work that I did under the
> PyHarness project where I was kind of working with interactively building a presentation along
> with an agent. But, you know, beyond that, I want an interactive surface for the agent and the
> human to draw diagrams and then adjust and comment have a conversation about them.
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

## What it asks for

1. **An interactive diagram surface** (in the human web UI, essy) where the agent draws a
   diagram and the human and agent discuss it.
2. **The human comments** on the diagram, with the same comment flow as x8jt.
3. **The human edits the diagram**, and the edit goes back to the agent as a diff, along with
   the new diagram, so the agent knows what changed.
4. **Simple and custom** is fine; no need for a full diagramming tool. Draw.io was tried in
   another project and didn't come out well.
5. **Prior work:** the PyHarness project, building a presentation interactively with an agent,
   and some diagram work.

## Notes (advisor)

- Later than x8jt: the human called the visualization part later work.
- Keeping x8jt's plain-text rule would point at a text diagram format kept in the document
  (e.g. Mermaid or D2 in a fenced block), so the diff is a text diff and the file reads without
  the UI. Not decided.
