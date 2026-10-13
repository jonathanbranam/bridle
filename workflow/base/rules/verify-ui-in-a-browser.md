---
id: verify-ui-in-a-browser
severity: must
roles: [manager, worker]
---
A task that changes what a user sees in a browser is not done until you have used it in a
browser. Start a throwaway site (own port, own data, own login: never the human's running
instance, never their database), log in, click through each acceptance item, and say in your
done message what you did and saw. Stop what you started, by pid. For a bridle project:
`bridle dev site up`; for another, the project's recipe (CLAUDE.md, "Verification").

- **The worker** does this before reporting done, one line per acceptance item.
- **The merger (manager)** does it again before merging: bring up your own throwaway site from
  the branch and click through the acceptance items yourself, as you run the check yourself
  rather than take the worker's word. A done message without the worker's line is sent back.
- If the browser tool is missing, rule `missing-tools` applies: ask, don't skip.

Why: the human, 2026-10-05: "While building a UI, the agents need to be able to do this
as well to verify that tasks are actually working." And 2026-10-12: "definitely before
something's merged, somebody needs to be able to click around on the site."
