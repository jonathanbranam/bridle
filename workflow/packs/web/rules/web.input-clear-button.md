---
id: web.input-clear-button
severity: should
roles: [worker, reviewer]
---
Input fields that accept user text should have a visible, accessible clear
button (often an X icon or button) to reset the field. For search or filter
input boxes, a clear button is mandatory.

```html
<!-- Good: input with clear button for a filter -->
<div class="input-wrapper">
  <input
    type="text"
    id="filter"
    placeholder="Filter results..."
    aria-label="Filter search box"
  />
  <button
    type="button"
    aria-label="Clear filter"
    onclick="document.getElementById('filter').value = ''"
  >
    &times;
  </button>
</div>
```

Why: users often need to reset an input to start over, especially in search
and filter contexts where the button is always needed. Without it, the user
must select all text and delete, adding friction. A clear button reduces
cognitive load and improves usability, particularly on mobile where text
selection is slow.
