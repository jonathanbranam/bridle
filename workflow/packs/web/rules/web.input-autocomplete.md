---
id: web.input-autocomplete
severity: should
roles: [worker, reviewer]
---
Text inputs should have an `autocomplete` attribute that matches their purpose:
`email`, `password`, `current-password`, `new-password`, `username`, `url`,
`tel`, `cc-number`, or `off` if autocomplete is inappropriate.

```html
<!-- Good -->
<input type="email" name="email" autocomplete="email" />
<input type="password" name="password" autocomplete="current-password" />
<input type="text" name="username" autocomplete="username" />

<!-- Less helpful -->
<input type="email" name="email" />
<input type="password" name="password" autocomplete="off" />
```

Why: autocomplete helps password managers work correctly, saves users typing on
mobile, and is a signal that the browser can use for security (e.g., blocking
autofill on `new-password` if the site is being spoofed). See track-web's
`packages/auth/src/LoginPage.tsx` for a working example.
