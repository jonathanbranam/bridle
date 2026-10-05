---
id: mobile.touch-action
severity: should
roles: [worker, reviewer]
---
Add `touch-action: manipulation;` to the `body` in CSS to prevent iOS Safari
from automatically zooming when an input receives focus.

```css
body {
  touch-action: manipulation;
  /* ... rest of styles */
}
```

Why: iOS Safari's default behaviour is to zoom when a focused input has
font-size under 16px. Setting `touch-action: manipulation` tells the browser
that the page handles gestures itself and prevents this automatic zoom. This
rule works alongside `mobile.viewport-meta` (which sets `maximum-scale=1`) to
ensure the page never zooms unexpectedly on mobile input focus. See track-web
commit 8f4793c.
