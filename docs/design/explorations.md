# Explorations: deliberately divergent, and known to be

> **Status (checked 2026-10-03):** Built and in use: the locked base rule `explorations`, which every spawned agent gets in its system prompt · Built, not wired in: the exploring agent's paragraph (`bridle prime worker --task <id>`; workers don't run prime, so an explore worker isn't told), `bridle explore check|new|conclude|abandon`; no project has a `design/explore/` tree yet · Planned: the path-scoped copy of the rule, `explore adopt`, the gate exemptions, keeping exploration code off main (not enforced)

A spike or exploration is a task of kind `explore`. It is **meant** to
contradict the accepted goals, architecture or specs, because its purpose is to
try something different. The recurring failure is that other agents come across
its artifacts, are surprised, and report the divergence as a defect. The design
deals with this up front:

- **The exploring agent is told at prime.** `bridle prime` for an `explore` task
  says (built in prime; nothing runs prime for a spawned worker yet): you are exploring an alternative; contradicting the architecture, goals
  and specs is intended; don't bring the code back into line and don't flag the
  divergence.
- **Every other agent is told too.** A locked base rule, delivered at every
  prime and as a path-scoped rule on `design/explore/**`, says: *material under
  `design/explore/`, or marked `exploratory: true`, deliberately contradicts the
  current design. It is not a defect, not stale, and not something to reconcile
  or report.*
- **Each exploration labels itself.** The findings doc starts with frontmatter
  bridle checks:

  ```yaml
  exploratory: true
  task: tw-e41a
  diverges-from: [a-12cd, r-7fa2]
  status: open            # open | concluded | adopted | abandoned
  ```

- **Explorations are exempt from checks built for mainline work**: the
  architecture gate, spec protection, impact conflicts against mainline tasks
  and trace validation. Locked safety rules (ports, dev servers, no pushing)
  **still apply**.
- **Exploration code never merges to main.** It stays on `explore/<id>`. Only
  the findings doc may merge, into `design/explore/<id>/`.
- **The human is not told about the divergence.** They hear about an exploration
  when it concludes with a recommendation. Adopting it goes through the normal
  path: an `arch-revision` and/or a goals change (human gates), then ordinary
  tasks on mainline. Until that happens, the exploration has no authority.

## Built

- **The locked base rule** `explorations` (`workflow/base/rules/explorations.md`, tagged
  for every role) carries the text above, so it is in every spawned agent's system prompt and
  `bridle prime` prints it to every role. The
  path-scoped copy on `design/explore/**` is not built: rules have no path scoping yet.
- **The exploring agent's paragraph**: `bridle prime worker --task <id>` looks the task up
  and, when its kind is `explore`, opens with the exploring paragraph. Without `--task`
  (prime is otherwise local) it is absent.
- **Frontmatter checking**: `bridle-spec`'s `explore` module requires all four keys,
  `exploratory: true`, a status in the set, and `diverges-from` ids that look like `g-`/`a-`/`r-`/`s-`
  ids. `bridle explore check [paths...]` (default `design/explore`, local) prints diagnostics and
  exits 1 on any error.
- **`bridle explore new|conclude|abandon <id>`**: `new` scaffolds `design/explore/<id>/findings.md`
  with status `open` (refusing to overwrite); `conclude` and `abandon` rewrite only the `status:`
  line, byte for byte otherwise.
- Not built: `explore adopt` (needs the arch-revision flow), and the gate exemptions.
