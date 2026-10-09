+++
id = "br-8z7j"
title = "Only the owner's clone can push the integration branch: enforced, not a rule"
kind = "feature"
state = "open"
created_at = "2026-10-09T18:08:52.862Z"
updated_at = "2026-10-09T18:39:58.147775Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "8z7j"
+++

docs/tickets/open/only-the-owner-s-clone-can-push-the-integration-branch-enfor-8z7j.md

## Thread

### note · external:orchestrator · 2026-10-09T18:10:19.792Z
Approval: the human via advisor/product-manager, 2026-10-09 ~2:05 PM ET (postmortem j7r4 comments c1-c5): 'it should be impossible for a different clone under bridle to push'. Design first: designer writes options into ticket 8z7j; the design goes back to the human before any build.

### note · agent:design-8z7j · 2026-10-09T18:34:58.387Z
Design options written into the ticket (commit d20d3f98): A pre-push hook owner check (recommended), B dead push URL, C credentials, D rule only. Awaiting the human's pick.

### note · external:advisor/product-manager · 2026-10-09T18:39:58.147Z
From advisor (product-manager): the human, 2026-10-09 ~2:50 PM ET, verbatim: 'Option A is good - note there is an open issue with the thoughtbots dotfiles/dotfiles-local that I use where they install some shared git hooks. I never use these and my shared hooks are all empty. I don't use them, ever. So, before this is implemented, I want that removed from dotfiles-local and then it needs to be pushed and cleaned up on both machines.' So: Option A (pre-push hook, owner check) approved. NOT BUILDABLE YET: blocked until the dotfiles-local shared git hooks (thoughtbot dotfiles' core.hooksPath / shared hooks) are removed, pushed, and cleaned up on dalek and the NUC; the PdM is coordinating that with the dotfiles-local aide on the NUC and will say here when it's done. Q2 (integration branch only vs also release) not answered; default to the recommendation (integration only) unless the human says otherwise. Designer's branch bridle/design-8z7j (d20d3f98) holds the design: please land that ticket edit on main.
