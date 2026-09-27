# Decided

1. **One machine drives.** All agents run on one machine and coordinate through
   it. Moving to a VPS or another machine must remain possible.
2. **Git is the source of truth.** Anything durable can be rebuilt from git
   remotes. Live coordination state may sit in a local database, but losing that
   database must lose nothing that matters.
3. **Rust.** Single static binary, `bridle`.
4. **OpenSpec is up for replacement, and so are all the skills.** Keep the
   workflow's shape; lose its sequencing and most of its human checks.
5. **The workflow, its rules and its guidelines live in one modifiable place**,
   as a shared base with layers on top: technology and solution packs, then per-project
   overrides and additions.
6. **Project knowledge is tiered** (added 2026-09-27): long-term goals with
   firmness and priority, a project-wide architecture that only the human can
   revise, detailed specs that trace up to it, and explorations that are allowed
   to diverge from all of it ([[docs/design/knowledge-tiers|knowledge tiers]]).
7. **Budget: one $100/month subscription, no overage** (added 2026-09-27).
   Token efficiency is a design constraint, and bridle manages usage limits:
   it picks models, pauses at limits, resumes at reset, and tracks token use
   over time ([[docs/design/usage-and-budget|usage and budget]]).
