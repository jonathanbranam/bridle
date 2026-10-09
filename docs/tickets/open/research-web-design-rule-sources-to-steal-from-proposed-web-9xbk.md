---
id: 9xbk
title: "Research: web design rule sources to steal from, proposed web/mobile pack rules, attribution, and shipping whole rule sets"
kind: research
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [22n2, g49c]
tasks: [br-9xbk]
---

## The ask

From the human, 2026-10-08 ~7:55 PM ET:

> "Is there a site with great design rules we can steal from? Do some searching. I've read these years back when I was working in web UIs. Send a subagent to research sites with strong clear guidance on web design principles and summarize what they find in a research ticket and specific suggestions on rules to add to that role."

Context: the rules live in bridle's `workflow/packs/web/rules/` and `workflow/packs/mobile/rules/`.
Today there are six web rules (aria-labels, form-labels, input-autocomplete, input-clear-button,
page-title, semantic-html) and five mobile rules (input-font-size, safe-area-insets,
text-selection, touch-action, viewport-meta). One more rule is already being filed: non-prose
inputs (username, email, IDs) get `autocapitalize="none" autocorrect="off" spellcheck="false"`.
The proposals below don't repeat any of these.

## Sources

| Source | URL | Best for | How concrete or checkable | Licence and reuse |
|---|---|---|---|---|
| **Vercel Web Interface Guidelines** | https://vercel.com/design/guidelines (repo: https://github.com/vercel-labs/web-interface-guidelines) | Short imperative rules for interactions, forms, animation, layout, content and performance. Written for agents too: it ships an `AGENTS.md` and a `/web-interface-guidelines` review command. | **Very high.** Most rules can be checked in a diff, e.g. "never `transition: all`", "don't block paste", "show errors beside their fields and focus the first one", "use `tabular-nums`". A few are matters of taste (shadow layers, optical alignment), and some are Vercel brand style (Title Case, "&"). | **MIT** (repo licence checked through the GitHub API). We can copy and adapt the text with attribution. |
| **GOV.UK Design System and Service Manual** | https://design-system.service.gov.uk/ , https://www.gov.uk/service-manual | Forms, error messages, error summary, buttons, plain-language content. Every component page has "when to use / when not to use" and research behind it. | **High.** E.g. errors go after the label and hint, use the same wording in the field and in the error summary, never clear fields on error, avoid "valid/invalid/please/oops", avoid disabled buttons. | Code **MIT**. Docs **Open Government Licence v3.0**, which allows reuse with attribution. |
| **WCAG 2.2 + WAI tutorials / Understanding docs** | https://www.w3.org/TR/WCAG22/ , https://www.w3.org/WAI/tutorials/ , https://www.w3.org/WAI/WCAG22/Understanding/ | The legal and accessibility floor, with numbered success criteria. | **Very high.** Testable by design: 1.4.3 contrast 4.5:1, 1.4.11 non-text contrast 3:1, 1.4.4 resize to 200%, 1.4.10 reflow at 320 CSS px, 2.4.7 / 2.4.11 focus visible and not obscured, 2.5.8 target size 24x24 px, 3.3.1 error identification, 3.3.8 accessible authentication (no paste blocking), 2.3.3 motion from interactions. | W3C Document License: cite and link; quoting with attribution is fine. |
| **U.S. Web Design System (USWDS)** | https://designsystem.digital.gov/ | Components, form templates, an accessibility checklist per component, and a "Form" pattern section. Similar to GOV.UK but American; strong on accessibility tests. | **High.** Each component has an "Accessibility tests" checklist. | US federal work, **public domain** in the US and CC0 elsewhere (per the repo LICENSE.md; the GitHub API reports NOASSERTION, so confirm before copying a lot). |
| **web.dev Learn (Forms, Accessibility, CSS) and Core Web Vitals** | https://web.dev/learn/forms , https://web.dev/learn/accessibility , https://web.dev/articles/vitals | How to implement: `type`, `inputmode`, `autocomplete`, `enterkeyhint`, validation APIs; CLS and INP thresholds. | **High** for code (the attribute to use); the Vitals thresholds are numbers (CLS < 0.1, INP < 200 ms, LCP < 2.5 s). | Content **CC BY 4.0**, code samples Apache 2.0. |
| **MDN Web Docs** | https://developer.mozilla.org/ | Reference for every attribute and media query (`inputmode`, `enterkeyhint`, `prefers-reduced-motion`, `font-variant-numeric`). | Reference, not rules. Use it to confirm facts. | CC-BY-SA 2.5 (prose); code samples CC0. |
| **Nielsen Norman Group** | https://www.nngroup.com/articles/ten-usability-heuristics/ , https://www.nngroup.com/articles/errors-forms-design-guidelines/ , https://www.nngroup.com/articles/web-form-design/ | The "why", backed by research: 10 heuristics, form errors, placeholder-as-label harm, link vs button, response-time limits (0.1 s / 1 s / 10 s). | **Medium.** Principles plus some concrete guidelines (e.g. no placeholder-only labels, errors next to fields). Not written as lint rules. | Copyrighted. Cite and link only; don't copy. |
| **Baymard Institute** | https://baymard.com/blog (e.g. inline validation, form field usability) | Large-sample e-commerce form and checkout usability findings. | **Medium-high** on forms (e.g. don't validate before the user leaves the field; keep errors adjacent). Much of it is paywalled. | Copyrighted, mostly paid. Cite free articles only. |
| **Luke Wroblewski, *Web Form Design* (2008) and lukew.com** | https://www.lukew.com/resources/web_form_design.asp | The classic forms book: top-aligned labels, primary vs secondary actions, inline validation. | Medium. Older, but the core findings still hold. | Book copyrighted; cite only. |
| **Smashing Magazine form-design articles** (Adam Silver's *Form Design Patterns*, etc.) | https://www.smashingmagazine.com/printed-books/form-design-patterns/ | Accessible form patterns that follow GOV.UK practice. | Medium-high. | Copyrighted; cite only. |
| **Apple Human Interface Guidelines** | https://developer.apple.com/design/human-interface-guidelines/ | Touch targets (44x44 pt), platform behaviour, motion, Dynamic Type. Native-first. | Medium. A few numeric rules; the rest is native-app guidance. | Copyrighted; cite only. |
| **Material Design 3** | https://m3.material.io/ | Touch targets (48x48 dp), states (hover/focus/pressed), motion tokens, text fields. | Medium. Specs are concrete but tied to Material's look. | Docs CC BY 4.0; code Apache 2.0. |
| **Refactoring UI** (Wathan and Schoger) | https://www.refactoringui.com/ | Visual hierarchy, spacing scales, grey-on-colour, de-emphasising secondary actions. | Low-medium for code review: mostly visual judgement. | Paid book; cite only. |
| **Butterick's Practical Typography** | https://practicaltypography.com/ | Line length (45-90 characters), curly quotes, real ellipsis, no double spaces, font size. | Medium. Line length and character choices are checkable. | Copyrighted, free to read (pay-what-you-want); cite only. |
| **Laws of UX** (Jon Yablonski) | https://lawsofux.com/ | Named principles (Fitts, Hick, Doherty threshold 400 ms, Jakob's law). | Low. Good for explaining a rule, not checkable in itself. | Copyrighted; cite only. |

## Recommended sources to steal from

1. **Vercel Web Interface Guidelines.** It's the closest in shape to our packs (short, imperative, checkable in a diff, already written for coding agents) and it's MIT, so we can lift rules nearly as written. Skip its brand-style section and its taste rules (shadows, optical alignment). Also skip "prefer APCA": WCAG 2 AA is the standard a reviewer can check today.
2. **GOV.UK Design System.** The best source for forms, errors and button behaviour, based on research with millions of users. The OGL allows reuse with attribution. It's strongest on wording rules for error messages, which nobody else covers as concretely.
3. **WCAG 2.2 (with the WAI Understanding docs).** The non-negotiable floor. Every "must" in our accessibility rules should cite a success-criterion number, so a reviewer has an objective test.
4. **web.dev Learn Forms / Accessibility (plus MDN as reference).** The "how" for each rule: which attribute, which media query. CC BY, so examples can be adapted.
5. **NN/g** as the source for the "Why:" lines (cite and link, don't copy). It gives the research behind the rules.

USWDS is a good alternative to GOV.UK (public domain, per-component accessibility checklists). Use it if we want US-flavoured wording.

## Proposed rules

22 rules: 18 web, 4 mobile. "must" means a WCAG failure or broken behaviour. "should" means strong practice.

| id | severity | statement | source(s) |
|---|---|---|---|
| `web.focus-visible` | must | Every focusable element shows a visible focus indicator (at least 3:1 against what's next to it). Never `outline: none` / `outline-0` without a `:focus-visible` replacement, and sticky headers or footers must not cover the focused element (use `scroll-padding`). | WCAG 2.4.7, 2.4.11, 1.4.11; Vercel (Interactions) |
| `web.color-contrast` | must | Text meets 4.5:1 contrast (3:1 for text 24px and up, or 18.66px bold and up). Icons, input borders and focus rings meet 3:1. Check each design-token pair, in both light and dark themes. | WCAG 1.4.3, 1.4.11; GOV.UK; USWDS |
| `web.not-color-alone` | must | Status, errors, required fields, and links inside prose aren't shown by colour alone. Add text, an icon with a label, or an underline. | WCAG 1.4.1; Vercel (Content) |
| `web.link-vs-button` | must | Navigation (changes the URL, can open in a new tab) uses `<a href>`. Actions use `<button type="button">` (or `type="submit"` in a form). No `<a>` without an `href`, no `onClick` on a `div`/`span`, no `<button>` that only navigates. | NN/g; Vercel ("use real links"); WCAG 4.1.2 |
| `web.input-type-inputmode` | should | Inputs use the right `type` (`email`, `tel`, `url`, `search`, `number` only for quantities) and `inputmode` (`numeric` for codes and IDs, `decimal` for amounts) so the right on-screen keyboard appears. Set `enterkeyhint` where the action is clear (`search`, `send`, `next`). | web.dev Learn Forms; MDN; Vercel (Forms); GOV.UK (don't use `type=number` for card numbers or dates) |
| `web.no-disabled-submit` | should | Don't disable a submit button because the form is incomplete or invalid. Let the user submit and show what to fix. Disable it only while the request is in flight, to stop double submits. | GOV.UK Button; Vercel (Forms); CMS Design System |
| `web.error-placement` | must | A validation error appears next to its field (after the label and hint, before the input), linked with `aria-describedby`, and the field gets `aria-invalid="true"`. On submit, move focus to the first invalid field (or to an error summary that links to the fields). Never clear the user's input when showing an error. | GOV.UK Error message / Error summary; WCAG 3.3.1, 3.3.3; Vercel; NN/g |
| `web.error-message-wording` | should | Error text says what happened and how to fix it, in the field's own words ("Enter your username", "Password must be 12 characters or more"). Not "Invalid input", "Error", "Oops", "This field is required", and no raw status codes or stack traces. | GOV.UK Error message; NN/g error guidelines; Vercel (Content) |
| `web.validate-on-submit` | should | Don't show errors while the user is still typing in a field. Validate on submit, or at the earliest on blur. Remove an error as soon as the user fixes it. Never block keystrokes (no `preventDefault` on key events to filter input). | Baymard inline validation; GOV.UK; Vercel ("allow any typing") |
| `web.button-labels-verbs` | should | Buttons say what they do, with a verb and its object ("Mark done", "Send answer", "Delete comment"), not "OK", "Submit", "Yes" or "Continue". In a confirmation dialog, the confirm button repeats the action ("Delete", not "Yes"). | NN/g; GOV.UK; Vercel (specific labels); Apple HIG |
| `web.loading-state` | should | Every async action has a pending state: the button keeps its label and adds a spinner or "Saving…". Long loads show a skeleton or indicator that appears only after ~150-300 ms (no flicker) and is announced through `aria-busy` or a live region. | Vercel (Interactions, Content); NN/g response-time limits |
| `web.empty-error-states` | must | Every list or data view renders three non-happy states: loading, empty (says why it's empty and what to do next), and error (says what failed, with Retry). No blank panels, and no spinner that never ends after a failed fetch. | Vercel ("design empty, sparse, dense and error states"; "every screen has a next step"); NN/g heuristic 9 |
| `web.live-region-feedback` | should | Toasts, inline status ("Saved", "Copied") and async validation messages go in a polite live region (`role="status"` / `aria-live="polite"`) so screen readers announce them. Use `role="alert"` only for errors that block the user. | Vercel; WCAG 4.1.3 |
| `web.reduced-motion` | must | Every animation or transition beyond a simple fade has a `prefers-reduced-motion: reduce` variant (Tailwind `motion-reduce:` / `motion-safe:`). Nothing autoplays or loops for more than 5 s without a pause control. | WCAG 2.3.3, 2.2.2; Vercel (Animations); web.dev; MDN |
| `web.no-transition-all` | should | Transitions list their properties (`transition-colors`, `transition-opacity`, `transition-transform`), never `transition: all` / `transition-all`. Animate `transform` and `opacity`, not `width`/`height`/`top`/`left`. | Vercel (Animations); web.dev rendering performance |
| `web.tabular-numbers` | should | Numbers that update in place or line up in columns (counts, timestamps, durations, table cells) use `font-variant-numeric: tabular-nums` (Tailwind `tabular-nums`), so digits don't shift. | Vercel (Content); Butterick (tabular figures) |
| `web.no-paste-blocking` | must | Never block paste, autofill or password managers on inputs (no `onPaste` `preventDefault`, no `autocomplete="off"` on login fields). One-time codes accept a pasted value. | WCAG 3.3.8; Vercel (Forms); NCSC / GOV.UK guidance on pasting passwords |
| `web.prose-line-length` | should | Running prose (documents, long descriptions, comments) caps its line length at about 45-80 characters (`max-w-prose` / `max-width: 65ch`) on the text block only, never on the page or shell. This makes concrete the bridle-ui convention "a page may cap line length of running prose only". | Butterick; WCAG 1.4.8 (AAA, 80 chars); USWDS typography |
| `web.reflow-no-hscroll` | must | At 320 CSS px wide (and at 200% zoom) the page has no horizontal scroll except inside things that need two dimensions (tables, code blocks, diagrams), which scroll in their own container. Long unbroken strings (IDs, URLs, paths) use `break-words` / `overflow-wrap: anywhere` or `min-w-0` on their flex child. | WCAG 1.4.10, 1.4.4; Vercel ("design for very long user content") |
| `web.destructive-confirm` | should | Destructive or irreversible actions (drop, delete, discard edits) either ask for confirmation in a dialog that names the action, or (preferred for quick actions) run at once and offer Undo for a few seconds. Warn before leaving a page with unsaved typed text (`beforeunload`). | Vercel (Interactions, Forms); NN/g heuristic 3 and 5; GOV.UK |
| `mobile.target-size` | must | Interactive targets are at least 24x24 CSS px (WCAG AA), and primary touch controls (buttons, row actions, checkboxes with their labels) are at least 44x44 px, by padding or a pseudo-element hit area if not visibly. Adjacent small targets have 8px or more between them. | WCAG 2.5.8 (2.5.5 AAA 44px); Apple HIG 44pt; Material 48dp; Vercel |
| `mobile.no-hover-only` | must | No information or action is reachable only by hover (tooltips with essential content, row actions that appear only on `:hover`). Touch has no hover: show it always, or on focus or tap. Wrap hover styling in `@media (hover: hover)` (Tailwind v4 does this by default) so taps don't leave sticky hover states. | WCAG 1.4.13; Vercel ("prefer inline help over tooltips"); Material |
| `mobile.gesture-alternatives` | must | Every swipe, drag, long-press or pinch action also has a single-tap button or keyboard equivalent. | WCAG 2.5.1, 2.5.7; Vercel (Interactions) |
| `mobile.autofocus-desktop-only` | should | Don't `autoFocus` inputs on page load on touch devices (it opens the keyboard and hides half the screen). Autofocus the primary input only on fine-pointer devices, or when the user just asked to type (e.g. opened a comment box). | Vercel (Interactions); NN/g |

(22 rules. Possible extras if wanted: `web.url-state` (filters, tabs and selected item live in the URL so refresh, Back and links work; Vercel), `web.heading-hierarchy-skip-link` (one `h1`, no skipped heading levels, a skip link; WCAG 1.3.1, 2.4.1), `web.color-scheme-meta` (set `color-scheme` and `theme-color` to match the theme; Vercel).)

## Notes

- **Conflict: `mobile.viewport-meta` is wrong and should change.** It requires
  `maximum-scale=1` and says it "disables zoom". That blocks pinch-zoom for low-vision users
  and fails **WCAG 1.4.4 Resize Text** (and Vercel's "don't disable browser zoom"). It also
  contradicts bridle-ui's own CLAUDE.md ("The viewport meta must not set `maximum-scale`"). iOS
  Safari has ignored `maximum-scale` for user zoom since iOS 10, so the rule mainly does harm on
  Android. Proposed text:
  `<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">`, plus
  "must not set `maximum-scale` or `user-scalable=no`". Rely on `mobile.input-font-size`
  (16px) to prevent focus-zoom.
- **`mobile.input-font-size`**: keep the 16px rule (Vercel and the bridle-ui convention agree).
  If its text or example suggests `maximum-scale=1` as an alternative or companion, drop that
  part for the same reason.
- **`mobile.touch-action` gets the reason wrong.** `touch-action: manipulation` removes
  double-tap-to-zoom (and the old 300 ms tap delay). It does **not** stop iOS zooming when an
  input gets focus; only the 16px font size does that. Vercel puts it on controls, not
  `body`. Putting it on `body` is acceptable, since pinch-zoom still works, but the "Why:" line
  should be corrected.
- **Contrast metric:** Vercel prefers APCA. We should stay with WCAG 2.x ratios for "must"
  rules because they're the legal standard and tools check them (axe, Lighthouse). APCA
  could be a later "should".
- **`web.input-autocomplete`** says to use `autocomplete="off"` where autocomplete isn't
  appropriate. `web.no-paste-blocking` should say that login and password fields never get
  `off`. Make the two rules agree.
- **`web.input-clear-button`** (clear button on every text input) is unusual: no major source
  requires it outside search fields. Consider making it "search and filter fields only".
- **Overlap check:** `web.link-vs-button` refines `web.semantic-html` (which mentions
  `<button>`/`<a>` in general). Keep it separate because a reviewer can check it directly, or
  merge it in as a sub-bullet. `web.error-placement` builds on `web.form-labels` and doesn't
  repeat it.
- **Licensing:** we can copy wording from Vercel (MIT), GOV.UK docs (OGL v3, with attribution),
  web.dev (CC BY 4.0) and USWDS (public domain). For NN/g, Baymard, Apple, Butterick, Refactoring
  UI and Laws of UX, only cite and link in the "Why:" lines.
- **Integration idea:** Vercel publishes its guidelines as an agent review command
  (`command.md` in the repo). A reviewer role could fetch it as a checklist, but frozen rules
  in our packs are more predictable and can be versioned. Prefer porting the rules.

## Attribution (the human, 2026-10-08 ~9:30 PM ET, via the bridle-ui aide, m-7409)

> Somewhere where we're building this, let's record the sources for attribution for everything. I think that's important. It's good that they're clean licenses, but let's be sure we're giving credit.

So: an attribution record in the packs (e.g. a SOURCES or ATTRIBUTION file per pack listing each source, URL and licence: Vercel MIT, GOV.UK MIT/OGL, web.dev CC BY 4.0, W3C, NN/g cite-only), plus a source line on every rule taken from one.

## Ship whole rule sets (the human, ~9:33 PM ET, m-7410; speech-to-text, "Rital" = bridle)

> The other comment I had was that it's fine to build extra rules that we're not turning on. I think if there's a well-established rule set, like Gov.uk, W3C, whatever, it'd be great to just ship all of those. We can just disable the ones that we don't agree with or we don't want to use, because that's part of how Rital works, right? We want to have things that users can turn on and off and not have to always build it themselves.

So: ship whole established rule sets (each with its attribution), with rules users can turn off.
