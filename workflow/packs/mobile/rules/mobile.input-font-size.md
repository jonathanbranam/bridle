---
id: mobile.input-font-size
severity: must
roles: [worker, reviewer]
---
All text inputs, textareas, and selects must have a font-size of at least 16px.

```css
/* Good: 16px minimum */
input, textarea, select {
  font-size: 16px;
}

/* Bad: too small; triggers iOS Safari zoom on focus */
input, textarea, select {
  font-size: 14px;
}
```

Why: iOS Safari (and some Android browsers) automatically zoom the page when a
focused input has font-size under 16px, making the interface unusable. Setting
the minimum to 16px, combined with `mobile.viewport-meta` (`maximum-scale=1`)
and `mobile.touch-action`, prevents this zoom. See track-web commit 8f4793c.
