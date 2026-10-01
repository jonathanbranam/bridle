---
id: sk7p
title: "`bridle token pair <machine>`: set up tokens between machines over SSH, like ssh-copy-id"
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [3ehu, k7mw, 9mxw]
---

## The ask

The human, 2026-10-01, verbatim: "I know agents shouldn't handle tokens, but can we also build
commands into bridle so that I can automate token setup between machines? Something like how SSH
copy keys command works? If I can successfully SSH to a machine, then it's safe to upload my public
key there. Similarly, if I'm authenticated on dalek, and I authenticate on NUC, I should be able to
run a bridle command to create tokens on both sides for humans and agents."

## Today

Cross-machine access (k7mw) works through `[<principal>.<machine>]` tables in
`~/.bridle/credentials.toml`. The human fills them by hand: run `bridle token create <name>
--print` on the daemon's machine, then paste the token into the other machine's file. Their own
CLI has no cross-machine token at all (3ehu). This has to be done for every principal (human,
orchestrator, advisor), every project and both directions.

## Proposal

`bridle token pair <machine>` (human only), run on one machine where the human is authenticated:

- **Trust comes from SSH**, as with `ssh-copy-id`. If `ssh <machine>` works, the human is
  authenticated on both sides, and the human's token on each daemon's own machine (the
  workspace token file) can mint there.
- **For each project whose daemon runs on the remote machine** (from the remote registry or k7mw
  config): over SSH, run the remote `bridle token create` for each wanted principal (`human`,
  `orchestrator`, `advisor`; `--for` to choose). Stream the tokens back on stdout and write them
  into the local `credentials.toml` under `[<principal>.<machine>]` (0600).
- **Then the same in the other direction**: mint locally for the projects on this machine, and
  write them into the remote `credentials.toml` by piping them to a remote `bridle` helper over
  SSH stdin.
- Tokens travel only over the SSH channel's stdin and stdout: never in argv (visible in `ps`),
  never in logs or printed output.
- Idempotent: an entry that already works (checked with a `GET /v1/status`) is kept. `--rotate`
  revokes it and mints a new one. Names stay unique per k7mw (`<name>@<machine>` for remote
  principals).
- **Agents never run it.** It's human-only like `token create`. The agents' Claude Code
  permissions deny `bridle token *` (check the session/worker deny lists).
- `--dry-run` lists what would be minted and written.

## Open questions

- The human's own cross-machine token: is it a `human` principal, or `external:human@<machine>`
  (it can't be `human` itself, which is minted from the workspace file)? Settle this together with
  3ehu part 1 (br-8b98), which adds the `[human.<machine>]` fallback this command would fill.
- Whether a remote `bridle` is on SSH's non-interactive PATH (`~/.cargo/bin`). Allow
  `--remote-bridle <path>`.

## Done when

On dalek, `bridle token pair nuc` leaves the human, the orchestrator and the advisor able to reach
every project's daemon on both machines, with no token copied by hand. A second run changes nothing.
