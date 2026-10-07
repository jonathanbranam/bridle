---
id: web.page-title
severity: should
roles: [worker, reviewer]
---
The page title (`document.title`) names what the user is viewing, so a browser
tab is identifiable from its title alone among many open tabs. A crowded tab bar
shows only the first 10-15 characters, so the order matters.

- **Name first.** The open thing's name (document name, ticket title, task ID and
  title) comes before anything else.
- **Type marker.** A short icon or emoji in front of the name says what kind of
  thing it is (to-do, task, ticket, spec, document). It replaces a "- Task -"
  word, which costs width. Use a distinct marker per type, and keep each type's
  marker the same everywhere.
- **Project after the name.** It is short. Leave it out where the name already
  says it (task IDs carry the project prefix).
- **Page name only when nothing is selected.** "System" and "Tasks" name the tab
  when there is no open item. While the item is loading, fall back to the page name.
- **The favicon carries the rest.** The app and the machine go in the favicon, not
  the title: for example the app's mark tinted with a colour derived from the
  machine name, set at runtime (an SVG data URL needs no per-machine assets).

The title updates on every route change and every selection change (opening a
document, task, or ticket), not only on page load.

Format: `<marker> <name> - <project>`; with nothing selected, `<Page> - <project>`.

Examples (the marker is shown as [type] here; use a real icon or emoji in the app):
- `[document] human-web-ui.md - bridle`
- `[ticket] Browser tab titles name what you're viewing - bridle`
- `[task] br-qbbk Browser tab titles`
- `Tasks - bridle`
- `System - bridle`

```javascript
const MARKERS = { todo: "[todo]", task: "[task]", ticket: "[ticket]", spec: "[spec]", document: "[document]" };

// Run in a route or selection effect, not only on load.
function updatePageTitle(item, pageName, project) {
  if (item) {
    // item = { type, name }; name first, project after
    document.title = `${MARKERS[item.type]} ${item.name} - ${project}`;
  } else {
    // nothing selected, or still loading: the page name
    document.title = `${pageName} - ${project}`;
  }
}
```

Why: the human, 2026-10-06: "The browser title should be updated when the user
has selected something in a route to show the name of what they are viewing. I
have 10 tabs that all say 'bridle' - I can't find my document or task or system
tab." And, the same evening: "title bar should include lots of info, but we need
to plan it carefully to fit on a crowded toolbar ... that won't all fit; can we
indicate machine and/or bridle with favicon? that would help. Prefer the 'name of
the open thing' where possible, maybe an icon / emoji to tell if it's a todo,
task, ticket, or spec?"
