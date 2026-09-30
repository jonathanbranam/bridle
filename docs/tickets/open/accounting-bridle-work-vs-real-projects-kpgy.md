---
id: kpgy
title: "Accounting across projects: how much goes to building bridle vs real projects"
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [xypj, y3sd, ksn2]
---

## The ask

The human, verbatim (2026-09-28):

> I want to somehow have a way to keep track of work across projects and keep an accounting
> of what Bridal is doing between different projects, just for reporting's sake. What I want
> to find out is how much token budget I am spending and wall time I am spending just
> building the tool versus actually building real projects.
>
> I think if each project stores that kind of history and keeps track of which components,
> tasks, and token budget were spent on, then we can coordinate that between the repos at
> some point in the future and get a sense of that.
>
> Agent swarms and software factories like this often get stuck spending a lot of time
> fixing themselves and addressing little bugs instead of pushing forward and actually
> adding new useful features. I think I'm getting stuck in that loop a little bit right now.
> I want to press forward with the main work of getting these projects onboarded. As we file
> these other tickets that are interesting and helpful, let's be sure we keep our eyes on the
> prize of actually using Bridal for very productive work.

## Notes

- Designed, not built: the usage ledger's project, task, kind and workflow-revision
  columns, and `bridle usage task`/`--by project`/`--by kind`
  (`docs/design/usage-and-budget.md`, "Tracking token use over time"). Today the turns
  ledger has per-agent tokens, cost, busy time and wall time; `bridle usage --by
  role|model|agent --since`.
- Each project runs its own daemon, so the per-project split already exists daemon by
  daemon (`bridle usage --project <name>`); what's missing is per task and component
  (y3sd), and one report across daemons (compare xypj, which shares a budget across them).
- The human's priority in the same message: onboarding real projects comes first; this is
  for reporting.
