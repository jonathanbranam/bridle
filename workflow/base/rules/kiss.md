---
id: kiss
severity: should
roles: [orchestrator, manager, worker, reviewer]
---
Keep it simple. Do what the task needs, to the precision it needs, and stop.

- **Match rigour to the stakes.** Correctness of the daemon, containment,
  merges, and the account-wide usage guard (the budget governor: hold, wind
  down, stop) must be right. Nice-to-haves need to work and be tested, not
  be exhaustive: usage breakdowns, cost audits and reports can be rough.
- **Don't over-verify.** `just check` once, green, is done for a worker. Don't
  add extra loops, repeated full test runs or bespoke benchmarks unless the
  task asks for them.
- **Don't gold-plate.** No extra flags, options, abstractions or edge cases
  beyond what the task names. Mention ideas in your report instead.
- **Use the smallest model that fits.** Light, mechanical work (docs, tickets,
  renames, small config or test fixes) runs on Haiku
  (`bridle agent spawn worker --model haiku ...`). Sonnet is for real design or
  tricky code.

Why: the human's words, 2026-09-27: "some agents have a tendency to overwork
and over verify things to be super thorough. Some parts of the work don't need
to be excessively thorough. Eg the usage tracking is a good nice to have but we
don't need super precise metrics around it. A general way to roughly track work
budget is all that is necessary. Being careful with the global usage is
important to get right however, since that is a limited and shared budget." And:
"If there is light work to do in a task, using haiku is a great way to save
token budget."

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
