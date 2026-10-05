---
id: web.aria-labels
severity: should
roles: [worker, reviewer]
---
Interactive elements that do not have semantic text content (buttons, links,
icon buttons) need an accessible label via `aria-label`, `aria-labelledby`, or
semantic context (a `<label>`). Native HTML elements like `<button>`, `<a>`, and
form controls provide accessible names automatically when they have text or an
associated label; use ARIA only where native semantics don't apply.

```html
<!-- Good: semantic text content -->
<button>Send</button>
<a href="/">Home</a>

<!-- Good: native label -->
<label for="search">Search</label>
<input id="search" type="text" />

<!-- Good: ARIA label when no text fits the design -->
<button aria-label="Close menu">
  <svg>...</svg>
</button>

<!-- Bad: icon button with no label -->
<button>
  <svg>...</svg>
</button>
```

Why: screen readers need a way to announce what a control does. Semantic HTML
(labels, button text, link text) is the first choice; ARIA supplements where
the design doesn't allow visible text.
