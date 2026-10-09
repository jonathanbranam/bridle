---
id: jrm2
title: "Document view: project dropdown, ticket search by ID, comment box at the highlight, full width"
kind: feature
opened: 2026-10-04
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [x8jt, v8kn, essy]
tasks: [br-jrm2]
closed: 2026-10-09T23:11:07Z
---

## The ask

The human, verbatim (2026-10-04, via advisor doc-review, looking at the Document page: two
unlabelled boxes and an Open button):

> I can't figure out the doc review UI at all. what are these empty boxes about?

and then:

> Please file that to be fixed. The project needs to be a dropdown with the available projects
> that it can connect to or that it knows about. The other one (I don't know what you put in
> there) needs to be a search box with inline search while I'm typing. I should be able to type
> just the ticket ID and have it open. If I paste a ticket ID and hit Open, that should work.
>
> Also, the comment box popped up way up top, and it doesn't fill my browser space. I'm going to
> use this a lot in the browser, not only on mobile, although supporting both is essential.
>
> When I highlight a section on mobile, or if the media width is too narrow, it should just cut
> the doc rendering in the center and open up the comment box directly below where I
> highlighted. If the width is wider and supports wider text, it should show up on the side,
> exactly like it will do in the send all this for implementation.
>
> Long term, this should be able to edit any file, and any document should be something I can
> give feedback and notes on. I'm not exactly sure what was hooked up and what was designed yet,
> but the priority is for open tickets for now. Long term, I should be able to edit and give
> feedback on any design doc.

("exactly like it will do in the send all this for implementation" was a transcription error.
The human, 2026-10-04, verbatim: "What I said was it should look exactly like Google Docs looks.")

## What's there now (advisor, checked 2026-10-04)

- `bridle-ui` `src/Document.tsx` (ui-acf0, ui-c39e): a form with two `<input>`s that have only
  `aria-label`s (`Project`, `Path`), no visible label or placeholder, and an Open button. Path
  is a repo-relative file path, e.g. `docs/tickets/open/<slug>-<id>.md`. Nothing lists or
  searches files.
- The new-comment box (`pending`) renders above the whole document, not by the selection.
  Threads already sit in a right-hand column (`grid-cols-[3fr_2fr]`) beside their block at every
  width.
- The page is capped at `max-w-3xl` (`src/App.tsx`), so on a desktop browser the document and
  margin share about 768 px.
- Gateway (`crates/bridle-gateway`): `GET /api/v1/projects` already lists the projects it knows,
  so the dropdown needs nothing new. The only document routes are `GET`/`PUT
  .../documents/{path}` on one exact path, so search and ticket-ID lookup need a gateway route.

## What it asks for

1. **Project is a dropdown** of the projects the gateway knows (`GET /api/v1/projects`).
2. **Path becomes a search box with inline results as you type.** Typing or pasting just a
   ticket ID (e.g. `x8jt`) and pressing Open opens that ticket. Open tickets come first. Other
   docs come later (item 6).
3. **Use the browser's width on a desktop.** No `max-w-3xl` cap on the document view.
4. **The comment box opens where you highlighted:**
   - On a narrow screen (mobile, or a narrow window), split the document after the highlighted
     block and open the comment box right below it, inline.
   - On a wide screen, it should look **exactly like Google Docs**: the comment box and threads
     sit in a right-hand margin beside the document, each level with its highlighted text.
5. **Desktop and mobile are both essential.**
6. **Long term, not now:** edit any file, and give feedback and notes on any document,
   especially design docs. Open tickets come first. The gateway's write route already takes any
   text file in the repo. Editing in place is
   [[everything-readable-and-editable-through-the-daemons-file-ba-v8kn|v8kn]] and the
   CodeMirror step in bridle-ui's plan.

7. **Commenting in the UI puts the document under review by itself** (the human, 2026-10-04,
   verbatim, via advisor doc-review):

   > If I use the user interface to add comments to the doc, that should automatically set the
   > document for review without me doing anything. If I edit it by hand, I guess it makes sense
   > that I have to add it for review, because I don't think we want something scanning the
   > entire file system for documents that need reviews. That's a TBD, maybe to consider.

   - A save from the UI that adds a comment thread runs the equivalent of `bridle review add`
     for that path, unless it's already under review.
   - Comments added by hand still need `bridle review add`. Nothing scans the repo for them.
     Whether anything should is TBD and out of scope here.
   - Today you have to add it by hand first, and "Request review" returns 400 on a document
     that isn't under review (`docs/design/human-web-ui.md`). Found on 3haz.

## Likely shape (advisor; for the planner, not decided)

- Gateway: one search route per project, e.g. `GET /api/v1/projects/{project}/documents?q=`,
  returning matching repo-relative paths. A bare ticket ID resolves to `docs/tickets/*/*-<id>.md`
  (spikes too). Start with open tickets and `docs/`, and widen it later.
- bridle-ui: a `<select>` from `/projects`, a combobox over the search route, a responsive
  comment box placed by the selection, and a wider layout for the document page.
- Auto-review (item 7): after a document `PUT`, if the saved text has a pending thread, the
  gateway asks the daemon to add the path to review, with the human's token as the review route
  already does. Pending uses the same rule as the daemon (`doc_watch::pending_threads`): the
  thread's newest entry is the human's and has no `· sent` mark. That's decided by the text
  alone, so the same file always gives the same answer. Adding a path that is already under
  review changes nothing. The UI changes nothing for this.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
