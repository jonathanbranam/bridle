---
id: c8qw
title: Bridle watches the remote CI and reports failures
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [3abb0af, b51d9b4]
specs: []
needs: []
see: [g3ck, mt7r, n6gy]
---

## The ask

The human, verbatim (2026-09-28):

> We really have to get away from running this just twice. [...] If we're running that three
> times for every merge, I can't imagine what that is accomplishing.
>
> Could we have a CI watching capability that Bridle can run, observe the remote CI, and then
> just send a message if it fails or look into a webhook? [...] Let's try to keep it simple
> [...] Just GitHub Actions is the only thing we need.
>
> Also, I'm thinking about the transition to my Intel NUC, and that little box is not going to
> handle all of these massive builds and CI runs.

## Where `just check` runs today, per merge

1. The worker, on its branch with `main` merged in: once, before it hands off. This is the
   gate, since CI only runs after the push.
2. The orchestrator: twice on `main`. **Stopped 2026-09-28**; the orchestrator now reads the
   CI result (`workflow/base/roles/orchestrator.md`).
3. GitHub Actions, on every push to `main`: on ubuntu-latest and macos-latest. It runs on
   GitHub's machines, not the laptop, and it's the only run that catches environment leaks
   such as g3ck (the global `init.defaultBranch`).

The manager doesn't run it. So what's left locally is one run per merge, by the worker.
mt7r (`just check-affected`) can shrink that further for the NUC.

## Proposal (KISS)

A small poller in the daemon, not an agent, using the `gh` CLI (already installed and
authenticated for the human; no webhook, no new dependency):

- **Opt-in**, per project: `[ci] github = true`, or detect a GitHub `origin` and `gh` on
  PATH. Without it, nothing runs.
- **When:** after the integration branch's tip changes on `origin` (the daemon sees the
  merger's push, or checks `git ls-remote` every few minutes), look up the run for that sha:
  `gh run list --commit <sha> --json databaseId,status,conclusion,url`.
- **Poll** every minute or two until `status == completed`, then drop it. Stop after about
  an hour.
- **Report:** emit a `ci.completed` event (sha, conclusion, url). The watcher and TUI see it.
  On a failure, send a note to the project's merger role (the manager), with the url and
  the failed jobs' names: "CI failed on <sha>: <jobs>; don't merge until it's green". Send
  nothing on success.
- **Status:** `bridle status` shows the last CI result for the integration branch.

Later, only if polling proves too slow or costly: a GitHub webhook
(`workflow_run` completed). That needs a reachable URL (e.g. Tailscale Funnel) and a secret,
which is more than today needs.

## Also worth deciding

- CI runs on every push, including doc-only commits (the orchestrator's tickets and state).
  A `paths-ignore` for `docs/**` and `*.md` in `.github/workflows/ci.yml` would skip those.
  It costs GitHub minutes, not the laptop, so this is optional.
- On the NUC: workers' one local run is the remaining load. mt7r (`check-affected`) for
  workers, with the full suite left to CI, is the natural next step.

## Resolution

Resolved by b51d9b4 (3abb0af): with `[ci] github = true` the daemon polls GitHub Actions (`gh`) for the integration branch's tip, emits the result, messages the manager on a failure, and `bridle status` shows the last CI result. The answer lives in docs/design/agent-host/operating-model.md ("CI watcher") and docs/design/cli.md.
