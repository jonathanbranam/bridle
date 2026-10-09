---
id: g49c
title: "Rule packs for the web and for mobile: standard forms and ARIA, and no zoom on mobile (track-web's fix, written down once)"
kind: feature
opened: 2026-10-04
repos: [bridle, bridle-ui, track-web]
changes: []
specs: []
needs: []
see: [t4rf]
tasks: [br-g49c, ui-g49c]
closed: 2026-10-09T23:11:02Z
---

## The ask


The human, verbatim (2026-10-04, to the bridle-ui aide, right after filing
[[bridle-ui-the-login-form-follows-web-standards-so-password-m-t4rf|t4rf]]; dictated, "Bridal"
is bridle):

> Also, we fixed this over on Track Web as well. This is the kind of stuff that needs to get into
> rules. This needs to be a rule. This is a rule that applies for a project that is building stuff
> on the web. You know, add ARIA tags, add used standard form labels, you know, all this stuff.
> I'm kind of surprised we need to have a rule for it anyway. Also, for web pages that need to
> work on mobile, which is probably all of them, but especially like the ones that I'm interested
> in building here, we want to be fully mobile friendly. There's this issue where if you need to
> make sure that the font size is set to something particular and you need to like turn off zoom
> in some certain way. So this needs to be a rule that can be followed and can be used in any
> project that's building like a PWA type of thing or any sort of native mobile. Again, we solved
> this in Track Web. This is one of the main reasons for the rule system and guidelines that we're
> developing with Bridal is that I spent hours fixing this once. I don't ever want to fix this
> again. This needs to be something encoded in a workflow system. So when I say make a web page
> that works on mobile, it always does this. Anyway, so go check on Bridal or check on Track Web
> rather. I think the answer has something to do with never setting the font size less than 11
> and turning off scale in various ways so that the UI doesn't zoom when you're using it.

## What track-web did

The mobile zoom fix is track-web commit `8f4793c` (2026-05-18, "Fixes zoom issues on input
controls"), still in place across its `client-*` apps:

- The viewport meta tag in each `index.html` becomes
  `<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1, viewport-fit=cover" />`
  (two of the ten clients still lack `maximum-scale=1`).
- Each app's `index.css` gets `body { touch-action: manipulation; }` under the comment
  "Prevent iOS auto-zoom on input focus".
- Every text input goes from Tailwind `text-sm` (14px) to `text-base` (16px). iOS Safari zooms
  the page when a focused input's font is under 16px, so the floor is 16px for inputs. That's
  the size behind the human's recollection of a minimum font size.
- `client-*/src/index.css` also sets the safe-area insets (`env(safe-area-inset-top)`, ...) for
  the notch and home indicator.

The working login form (standard labels and autocomplete) is track-web's
`packages/auth/src/LoginPage.tsx`; see t4rf.

## Context

- Rules live in bridle's `workflow/`: base rules in `workflow/base/rules/`, and opt-in packs in
  `workflow/packs/<name>/` (python, typescript, vim today), which a project enables with
  `packs = [...]` in `.bridle/config.toml`. bridle-ui and track-web both use `packs = ["typescript"]`.
- bridle-ui's `index.html` at 515ad47 has
  `<meta name="viewport" content="width=device-width, initial-scale=1.0" />` (none of the fix).

## Two more rules from the human (2026-10-04 ~8:50 PM ET, to the bridle-ui aide)

The human, verbatim (dictated):

> These are rules that need to get pushed back into the bridle design.
> - I think just UI design: most of the time, an input box should have a clear button. If it's a
>   search box of some kind, that's not a hard and fast rule yet. It depends on the context. For a
>   whole form, we have a clear or something box, but for the kind of search box that we have here,
>   filtering for a bunch of things, you always have to clear that.
> - For the mobile rollup, we need to have a rule that talks about how to handle selection properly
>   on mobile. If we're writing a task that requires selecting text on mobile and interacting with
>   it in a specific way, that's handled properly.

Where these came from in bridle-ui tonight:

- Clear button: [[bridle-ui-a-clear-x-button-on-the-document-page-s-search-box-wu7r|wu7r]]
  (ui-7mcp, 44e8faf). The page's own button with `aria-label="Clear search"`, shown when the box
  has text; tapping it empties the box and keeps focus. A native `type="search"` clear isn't shown
  on iOS Safari, so it can't be relied on.
- Mobile selection: [[bridle-ui-highlight-to-comment-doesn-t-trigger-on-mobile-onl-c2xn|c2xn]]
  (ui-s3xe, c5c0a0a). Highlight-to-comment listened only for `mouseup`, which a phone's native
  selection (long-press, drag the handles) doesn't fire. The fix acts on `selectionchange`,
  debounced (~300 ms after the selection settles).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
