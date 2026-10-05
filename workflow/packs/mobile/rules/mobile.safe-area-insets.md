---
id: mobile.safe-area-insets
severity: should
roles: [worker, reviewer]
---
On notched or rounded-corner devices (iPhones, Android flagships), use CSS
safe-area inset variables to keep content out of the notch and home indicator.

```css
/* Apply to fixed or sticky elements at the edges of the viewport */
header {
  padding-left: max(1rem, env(safe-area-inset-left));
  padding-right: max(1rem, env(safe-area-inset-right));
  padding-top: max(0, env(safe-area-inset-top));
}

footer {
  padding-left: max(1rem, env(safe-area-inset-left));
  padding-right: max(1rem, env(safe-area-inset-right));
  padding-bottom: max(1rem, env(safe-area-inset-bottom));
}
```

Why: on notched devices, system UI (status bar, notch, home indicator) overlaps
the viewport. The `env(safe-area-inset-*)` CSS variables tell you how much to
inset content to avoid them. Combine with `mobile.viewport-meta`'s
`viewport-fit=cover` to use the full screen. See track-web commit 8f4793c.
