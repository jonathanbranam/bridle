---
id: 2mtr
title: Workers report missing tools and failed fetches plainly; a research task went to a worker with no web tools
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-2mtr, br-sfpg]
---

## The ask

From an incident on track-web, 2026-10-04 (logged in `docs/context/incidents.md`). The human,
verbatim (~4:30 PM ET, via track-web's aide):

> Ask the bridle aide to file an incident and investigate. Send as much information as you can such
> as the request for research and what sites were reached and if possible any that weren't. Ask
> that prompt for workers are updated to record these failures and report them

## What happened

track-web task tw-sxfh (ticket h679, Hole.io research) went to worker `holeio-research` (sonnet,
medium, Claude Code 2.1.289): "Research Hole.io gameplay and player reviews via web research, and
write the proposed rule set into ...h679". The worker reported "web search and the game wiki were
unreachable, so exact numbers (growth curve, level count, bot tuning) are not sourced". The
orchestrator relayed it to the human as "web search was partly unreachable".

The network was fine. **The worker had no web tools.** track-web's `[roles.worker]` has
`allowed_tools = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]`. That's the same list as
bridle's own `.bridle/config.toml` and the default in
`docs/design/agent-host/roles-and-config.md`, so every project copied from it is the same. The
transcript never mentions WebSearch or WebFetch: the tools weren't offered. The worker fell back to
`curl` in Bash and didn't say a tool was missing.

Sites, all via curl:

- Reached: `en.wikipedia.org/w/index.php?title=Hole.io&action=raw`;
  `itunes.apple.com/search?term=hole.io...` (id 1389111413, 4.59 stars, 1.97M ratings);
  `itunes.apple.com/us/rss/customerreviews/.../id=1389111413/...` (~100 reviews used);
  `apps.apple.com/us/app/hole-io/id1389111413`; `hole-io.com` (status only; the human had given
  `holeio.com`, never fetched).
- Failed: `holeio.fandom.com/wiki/Hole.io_Wiki` (Cloudflare "Just a moment..." challenge);
  `www.ign.com/wikis/hole-io/Tips_and_Tricks` ("...looked a bit suspicious, and we block
  suspicious stuff"); two `web.archive.org` copies of the IGN page (no usable text); an iTunes feed
  with the wrong app id (agent error, corrected).
- Never tried: any general search, so no review sites, Reddit, YouTube or articles.

Transcript: `~/.claude/projects/-Volumes-Data-work-track-web-workspace-wt-holeio-research/f3441839-4621-4a7b-a0d9-ab83e9d8e7e4.jsonl`.

## Asks

1. **The human's ask: workers record and report these failures.** `workflow/base/roles/worker.md`
   (or a base rule) says: when a tool the task needs is missing, or a fetch or site fails, record
   each one (what was needed, the URL or tool, the exact error) in the task thread and the summary,
   and tell the manager, instead of a vague "unreachable". A missing tool the task depends on is a
   blocker to raise (`bridle send <manager> --question`), not a footnote. The manager and the
   orchestrator pass such failures up as failures, not as asides.
2. **Suggested by the aides, not yet the human's decision: research tasks get web tools.** Options:
   (a) a `researcher` role (or a worker variant) with `WebSearch` and `WebFetch`, and research
   tasks assigned to it; (b) add both to the default worker; (c) the manager checks a task's needs
   against the role's tools before spawning and refuses a research task for a role without web tools.
   (a) plus (c) seems safest: workers keep a small tool set, and the gap can't recur silently.

## The human's decision (2026-10-04 ~8:50 PM ET, via the aide)

> For 2mtr yeah, that makes sense. yeah, I like your suggestion to go with both A and C. That seems
> very reasonable. There's not an appropriate worker to find the tools needed, and the manager can
> complain to somebody, the orchestrator, I guess.
>
> We have some pretty good information on SXF already, but might as well go ahead and rerun it.

So: (a) a `researcher` role with `WebSearch` and `WebFetch`, plus (c) the manager checks a task's
needs against the role's tools before spawning; when no role has them, the manager tells the
orchestrator. Re-run track-web's tw-sxfh (with the researcher role). How a re-run in another project
gets sequenced after this lands: [[cross-project-task-dependencies-a-task-waits-on-another-proj-yug5|yug5]].
