---
id: web.semantic-html
severity: should
roles: [worker, reviewer]
---
Use semantic HTML elements (`<button>`, `<a>`, `<header>`, `<nav>`, `<main>`,
`<section>`, `<article>`, `<form>`) instead of generic `<div>` or `<span>` with
ARIA roles, when the semantics match the content.

```html
<!-- Good: semantic elements -->
<button>Send</button>
<a href="/">Home</a>
<nav>
  <ul>
    <li><a href="/">Home</a></li>
  </ul>
</nav>
<form>
  <input type="text" />
</form>

<!-- Less accessible: div with role instead of button -->
<div role="button" onclick="...">Send</div>

<!-- Less accessible: div instead of nav -->
<div role="navigation">
  <ul>
    <li><a href="/">Home</a></li>
  </ul>
</div>
```

Why: semantic HTML works without JavaScript, is understood by all assistive
technologies, requires no maintenance of ARIA attributes, and gives browsers
built-in accessibility (focus management, keyboard navigation, etc.).
