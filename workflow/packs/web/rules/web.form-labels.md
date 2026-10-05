---
id: web.form-labels
severity: must
roles: [worker, reviewer]
---
Every text input, select, textarea, and button has an associated `<label>` with
a proper `for` attribute, or a `<label>` that wraps the control.

```html
<!-- Good: label with for -->
<label for="email">Email address</label>
<input id="email" type="email" name="email" />

<!-- Good: label wrapping input -->
<label>
  Password
  <input type="password" name="password" />
</label>

<!-- Bad: no label -->
<input type="email" placeholder="Email address" />
```

Why: labels make forms accessible to screen readers, improve touch targets on
mobile, and let users understand what each field is for. A placeholder alone
isn't a label and vanishes when the input has focus.
