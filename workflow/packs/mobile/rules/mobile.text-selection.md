---
id: mobile.text-selection
severity: should
roles: [worker, reviewer]
---
When a mobile page requires users to select and interact with text, ensure
selection is enabled and operates smoothly. Use CSS to allow text selection
on interactive elements that would otherwise suppress it.

```css
/* Allow text selection on mobile for interactive content */
.interactive-content {
  user-select: text;
  -webkit-user-select: text;
  -moz-user-select: text;
  -ms-user-select: text;
}

/* But disable on non-interactive touch targets */
button, [role="button"], .control {
  user-select: none;
  -webkit-user-select: none;
}
```

Why: on mobile browsers, text selection is often disabled on interactive
elements to prevent accidental selection when the user means to tap or
scroll. When your design or task requires users to actually select and copy
or highlight text, explicitly enable it with `user-select: text`. Test on
iOS Safari and Android browsers to ensure selection works smoothly without
interfering with touch interactions.
