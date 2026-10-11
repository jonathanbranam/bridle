+++
id = "br-r2nr"
title = "Rename the NUC: host and bridle machine name, on both machines (vega?)"
kind = "chore"
state = "claimed"
created_at = "2026-10-11T22:28:58.676Z"
updated_at = "2026-10-11T22:33:37.502383Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "human",
]
priority_at = "2026-10-11T22:28:58.678105Z"
ticket = "r2nr"
+++

docs/tickets/open/rename-the-nuc-host-and-bridle-machine-name-on-both-machines-r2nr.md

## Thread

### note · external:advisor/wsl2 · 2026-10-11T22:28:58.678Z
created for the human, priority normal

### note · external:advisor/wsl2 · 2026-10-11T22:28:58.679Z
To-do for you (normal priority): Rename the NUC: host and bridle machine name, on both machines (vega?). Finish it with `bridle ticket done br-r2nr`.

### note · external:orchestrator · 2026-10-11T22:32:24.408Z
orchestrator: step 6 checked against the code (2026-10-11). Corrections, then the order to run.

## Corrections

1. MISSING, and it bites at step 1: the state branch's owner.toml and the pre-push hook use the
   Linux host name (`hostname`, crates/bridle-daemon/src/lib.rs machine_host), not
   `[machine] name`. After `hostnamectl set-hostname vega`, each NUC project daemon refuses to
   start at its next restart (owner.toml says nuc; exit 78, which the systemd unit treats as
   final), and the hook refuses integration-branch pushes from the NUC. Fix: once per NUC
   project, stop its unit and run `bridle --project <p> serve --take-over` (then start the unit
   again).
2. `[human.nuc]` (and the other `[*.nuc]` sections) in dalek's credentials are the CLI's lookup
   key: rename them in the same edit as dalek's `[projects] machine = "nuc"`.
3. Drop "re-run `bridle systemd install`": unit names are `bridle-<project>.service`, keyed by
   project, so nothing changes (and it errors on existing units without --force).
4. Re-mint commands need the global `--project` and, for visitors, `--home`:
   `bridle --project <dalek-project> token create --peer vega`
   `bridle --project <dalek-project> token create <role> --machine vega --home <nuc-project>`
   Don't use `token pair` for these (it mints visitors with no home, so replies don't come home).
   Re-minting `peer:nuc` is optional: /v1/forward accepts any peer token whatever its name.
5. Revoke LAST and later, not with the re-mint: a reply to an old `@nuc` message routes only while
   the `@nuc` principal exists; once revoked it fails "no such recipient". Revoke with
   `bridle --project <p> token revoke orchestrator@nuc` (and human@nuc, aide@nuc, advisor@nuc,
   peer:nuc) once the old threads are done.
6. No change needed: unit names, the daemon registry, remote watchers (keyed by project), and the
   `@dalek` visitors on the NUC. History (messages, each daemon's principals rows) stays.

## Order

1. Both machines: `bridle status`, every outbox empty (forward de-duplication is keyed by the origin
   machine, so a row retried across the rename could deliver twice).
2. On dalek: mint the `@vega` visitors (and `peer:vega` if wanted); paste them into the NUC's
   `[<role>.dalek]` / `[peer]` values. The old tokens keep working meanwhile.
3. Steps 1-2 (host name, Tailscale).
4. On the NUC, per project: stop the unit, `bridle --project <p> serve --take-over`, start the unit.
5. Config on both machines (step 3 plus step 6's config edits, dalek's credentials sections in the
   same go). Routing and the sender's machine label are read from config per send, so no daemon
   restart is needed for this; restart the NUC gateway for `public_url`.
6. Steps 4-5 (ssh, dotfiles).
7. Check: `bridle doctor` / `bridle daemons` on each machine; a message each way arrives and says
   `@vega`.
8. Days later: revoke the `@nuc` tokens (correction 5).

### note · external:orchestrator · 2026-10-11T22:33:37.502Z
orchestrator: step 4 answer: 'serve --take-over' runs the daemon until stopped; it doesn't exit after claiming (docs/design/cli.md, 'Owner refusal exit code': run it by hand once, stop it, then start the unit). The new owner.toml is written at the daemon's next state flush, so before Ctrl-C wait until 'bridle status --project <p>' shows the state line as pushed. Since the unit is stopped, the hand-run daemon serves alone; nothing else to do.
