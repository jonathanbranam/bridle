---
id: bnhn
title: "bridle-ui: render markdown, turn wiki links and file paths into links that open in the UI"
kind: feature
opened: 2026-10-04
repos: [bridle-ui, bridle]
changes: []
specs: []
needs: []
see: [jrm2, ehv6, x8jt, k3qx]
tasks: [br-bnhn]
closed: 2026-10-09T23:11:06Z
---

## The ask


The human, verbatim (2026-10-04, to the bridle-ui aide; dictated):

> Oh, the next thing that needs to happen is that links need to resolve in the browser. So, if I'm
> looking at basically anywhere a file shows up, I think for now it'd be just great if they, if we
> auto detect file names. That should be pretty straightforward. File names are kind of obvious.
> And should be easy to detect and link. If we want to be smarter about it, we could, you know, do
> something smarter. Bare file names, I don't know. Probably don't. I don't know if they should
> link or not. Like, readme would obviously link to the project's readme. That might be, like,
> unintentional. So, I don't know about that, but... If it's like a docs slash anything, then you
> should, the UI should check if that file exists on disk and then link it if it does. And then
> anything that's an actual like wiki link, like using my auto wiki link style, those should turn
> into regular links. Any markdown should render. So we should, you know, adopt a markdown renderer
> here. We're already using one somewhere. Maybe in track web somewhere. I'm using a markdown
> renderer. At any rate, do some research. I know I can't think of any other projects that would
> do that besides track web, but I definitely have I'm pretty sure I'm using one somewhere. But I
> can also do some research on appropriate package. For that. So it might be easiest to do the
> auto linking as a pre-step before handing it to the markdown render and just turn those file
> paths into markdown links. That seems like the best approach. And then, you know, adding a
> markdown render is going to make some of the highlighting and commenting. Interactions, you know,
> a little more work to implement. So you're gonna have to split the buffer, and you're gonna have
> to split the buffer intelligently based on how Markdown works. So be careful for that.

## Context

- The renderer used elsewhere: track-web's `client-trips/package.json` has `react-markdown`
  ^10.1.0 and `remark-gfm` ^4.0.1 (`/Volumes/Data/work/track-web-workspace/track-web`). No other
  track-web client has one.
- bridle-ui today (`src/Document.tsx` at 515ad47) renders documents by hand: "Only what the
  documents use: `code` and **bold**. A real markdown library isn't worth it yet." Comment
  highlights are drawn over that text ("Markup spanning a highlight edge ...", same file).
- The wiki link style in bridle docs (`docs/README.md`): `[[docs/design/gates|gates]]` for a doc
  by path, `[[<ticket-stem>|alias]]` for a ticket by its full stem (no folder); tickets move
  between `open/` and `resolved/`.
- The gateway already reads a document (`GET /api/v1/projects/{project}/documents/{*path}`) and
  searches them (`GET /api/v1/projects/{project}/documents?q=`), in
  `crates/bridle-gateway/src/documents.rs`. The open document's URL is
  `/document?project=..&path=..` (k3qx, ui-n6cu).

## Work (UI)

bridle-ui task ui-pmkd renders markdown and front matter and links targets for both bnhn and a3yd (orchestrator, 2026-10-04). It starts once br-bnhn and br-a3yd land.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
