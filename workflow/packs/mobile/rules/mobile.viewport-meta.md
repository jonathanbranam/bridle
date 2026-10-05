---
id: mobile.viewport-meta
severity: must
roles: [worker, reviewer]
---
Every HTML document must have a viewport meta tag in the `<head>` that disables
zoom and fits the page to the device width:

```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1, viewport-fit=cover" />
```

The attributes mean:
- `width=device-width`: render at the device's CSS pixel width (not zoomed out)
- `initial-scale=1.0`: start without zoom
- `maximum-scale=1`: prevent user zoom (combined with `initial-scale=1`, this
  disables the iOS Safari behavior of zooming on input focus)
- `viewport-fit=cover`: use the full screen on notched devices, accounting for
  safe-area insets (see `mobile.safe-area-insets`)

Why: without this, mobile browsers zoom the page when a focused input has
font-size under 16px, making the UI unusable. The viewport meta tag is the
standard, browser-supported way to prevent this zoom. See track-web commit
8f4793c and the `mobile.input-font-size` rule.
