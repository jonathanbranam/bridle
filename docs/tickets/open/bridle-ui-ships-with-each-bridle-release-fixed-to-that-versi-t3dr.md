---
id: t3dr
title: bridle-ui ships with each bridle release, fixed to that version, and self-upgrade moves both
kind: feature
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [fv86, tc7t, chvf]
tasks: [br-t3dr]
---

## The ask

The human, 2026-10-10 ~2:00 PM ET, verbatim (to advisor product-manager):

> What I expect from this, if it's not in this epic, we should add it, is that I can install
> bridle without doing a clone, and that can run and self-upgrade. It includes the bridle daemon,
> the bridle gateway, and, of course, the bridle-ui as part of that, all fixed to a version that
> comes from bridle. ... If the UI is not done, that could be a separate epic because I understand
> it's a different piece of work.

## Where it stands (PdM, 2026-10-10)

- The gateway serves `~/.bridle/ui/` and checks the UI build's recorded API version against its
  own: warn or refuse (`docs/design/human-web-ui.md`, task 8).
- bridle-ui is its own repo and project. Its build is put in `~/.bridle/ui/` by hand from a
  bridle-ui clone. A bridle release carries only the `bridle` binary, so a machine installed from
  a release (fv86) has no UI, and a self-upgrade doesn't move the UI.
- tc7t option C (br-785a, pending): a landing that needs a UI install shows without a manual step.
  It is about the dev machine following landings, not releases.

## The ask

The UI ships with bridle, fixed to a bridle version: a machine installed from a release gets the
UI that matches it, and a self-upgrade moves both together. For a designer pass: where the UI
build comes from (bridle's release workflow builds a pinned bridle-ui commit, or bridle-ui
publishes its own release that bridle names), how the pin is recorded, how the daemon installs it
into `~/.bridle/ui/` on upgrade, and how this fits tc7t on the dev machine.

## The human's requirements for the short term (2026-10-10)

The human, 2026-10-10 ~1:30 PM ET, verbatim (to the aide):

> And on the UI, I'm okay if that's a separate command, or if it's a separate tarball or a
> separate download, or if I'm going to send this somewhere else. Yeah, I can do that.
>
> For shipping the UI, I lost it. X, shoot. There's a ticket that says that the UI is shipped
> along with Bridle, upgraded along with Bridle on a machine that doesn't have a clone. I think
> it's going to go in a new epic for the product manager.
>
> What I want to say is that I'm okay if that's slightly separate, or I don't care if it's a
> `git clone`, a read-only clone or something, a shallow clone, if that's the way to do it. Maybe
> that's fine, or if it needs a local build of Node, a local compile, I don't really care that
> much. Long term, it should be kind of part of the Bridle installation, but if it's a separate
> thing, that's fine.
>
> What I really want is that the short-term deliverable does not require cloning the Bridle UI
> and setting up Bridle on it. It should also be automated, and it should be tied to the Bridle
> version so that the two stay in sync. Those are kind of the main requirements.

So, for the designer: **must** (short term) — no hand-made bridle-ui clone with bridle set up on
it; automated; tied to the bridle version so the two stay in step. **May** — a separate command,
tarball or download; a shallow or read-only clone made by bridle itself; a local Node build.
**Long term** — part of the bridle installation.
