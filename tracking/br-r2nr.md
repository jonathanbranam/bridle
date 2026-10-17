+++
id = "br-r2nr"
title = "Rename the NUC: host and bridle machine name, on both machines (vega?)"
kind = "chore"
state = "integrated"
created_at = "2026-10-11T22:28:58.676Z"
updated_at = "2026-10-17T11:37:09.518382Z"
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

### note · external:aide · 2026-10-12T01:49:50.715Z
The human, 2026-10-11 ~9:55 PM ET (to aide), verbatim: "can I go ahead with manual nuc rename? Renames are not going to be a common occurrence in proactice". Aide: yes, by the steps above; they don't depend on 2wnn/ga6s.

### note · external:aide · 2026-10-12T02:26:19.794Z
Step 0, before the rename: upgrade the NUC to v0.6.2 and turn on release self-upgrade. The human, 2026-10-11 ~10:15 PM ET (to aide), verbatim: "add them to the rename ticket instead. I am already working on them." Steps from the orchestrator's m-10737 (written for v0.6.1), with v0.6.2, checked by aide against scripts/install-release.sh and docs/design/agent-host/daemon.md "Release upgrade":

On the NUC (currently bridle 0.5.0 in ~/.cargo/bin):
  cp -p ~/.cargo/bin/bridle ~/.cargo/bin/bridle.0.5.0
  curl -fsSL -o /tmp/ir.sh https://raw.githubusercontent.com/jonathanbranam/bridle/main/scripts/install-release.sh
  BRIDLE_INSTALL_DIR=$HOME/.cargo/bin bash /tmp/ir.sh v0.6.2 && bridle --version   # 0.6.2
In ~/.bridle/config.toml: delete the first line (workflow = "/srv/shared/work/bridle-work/bridle/workflow"; a release daemon keeps its own workflow checkout at its tag) and add:
  [daemon]
  self_upgrade = "release"
Restart one at a time:
  for s in meta-notes notes dotfiles-local meta-notes-ui gateway; do systemctl --user restart bridle-$s; sleep 10; systemctl --user is-active bridle-$s; done
Verify the running daemons (not the binary):
  for p in meta-notes notes dotfiles-local meta-notes-ui; do echo "$p $(bridle status --project $p --json | jq -r .daemon.version)"; done
Caveat: the first start on 0.6.2 migrates each project's database, so bridle.0.5.0 is only a sure fallback before that; if a daemon won't start, read journalctl --user -u bridle-<p> -n 50 rather than rolling back.

### note · external:aide · 2026-10-12T02:27:21.205Z
Correction to step 0 (aide, 2026-10-11 ~10:30 PM ET): do NOT delete the machine-wide workflow line on the NUC. meta-notes' committed .bridle/config.toml sets workflow = "/Volumes/Data/work/bridle/bridle/workflow" (a dalek path) and meta-notes-ui's sets the /srv/shared path; the machine line was masking the first. With it removed, bridle-meta-notes crash-looped on 0.6.2 ('workflow directory ... missing or unreadable'). Keep workflow = "/srv/shared/work/bridle-work/bridle/workflow" until those project configs drop their workflow lines.

### note · external:aide · 2026-10-12T02:35:12.260Z
Step 0 addendum (aide, ~10:35 PM ET): also set release_repo = "jonathanbranam/bridle" under [daemon] in the NUC's ~/.bridle/config.toml, then restart the four daemons; without it the release repo and workflow fetch fall back to each project's origin (meta-notes etc.), so no bridle release is found and ~/.bridle/workflow stays empty.

### note · external:advisor/nuc-rename · 2026-10-12T03:17:28.211Z
advisor nuc-rename, 2026-10-11 ~11:30 PM ET: progress. Done by the human: steps 1, 3, 4 (all four NUC projects taken over as vesta), 6 partly. Interim instead of step 5: [machines] nuc = "vesta" on dalek and vesta (key stays nuc; [machine] name, [projects], credential sections and tokens still say nuc), gateway public_url on vesta, gateway restarted; dalek reaches the NUC projects. Also set on vesta: [daemon] release_repo = "jonathanbranam/bridle" (bug ajp5). Left: once the eurm fix is released and installed on both machines, the rest of step 5 (rename the machine to vesta everywhere at once), then bridle token pair --machines dalek,vesta --rotate in place of step 2, then steps 7 and 8. Related bugs filed: eas4 (serve --project vs cwd), ajp5.

### note · external:advisor/nuc-rename · 2026-10-14T13:51:33.024Z
advisor nuc-rename, 2026-10-14 (checked by advisor nuc-rename at the human's request): steps 0-7 are done.
- Step 0: both machines run 0.7.2; vesta has self_upgrade = "release" and release_repo set.
- Step 3: hostname is vesta; ~/.rcrc points to host-vesta/rcrc.
- Step 4: all four vesta daemons and the gateway are active.
- Step 5: done on both machines. [machine] name = "vesta"; [machines] and [projects] say vesta; credential sections are [*.vesta] on dalek and [*.dalek] on vesta; public_url is vesta.
- Step 6: no nuc entry left in dalek's known_hosts; host-vesta/ exists in dotfiles-local.
- Step 7: bridle doctor is clean on dalek (one .gitignore warning, not about the rename). On vesta it fails only because it ran outside a clone. bridle daemons is fine on both. Messages arrive each way, as system@vesta / aide@vesta on bridle and system@dalek on vesta.
Left: step 8. The bridle daemon still has external:orchestrator@nuc and external:advisor@nuc; the other dalek projects have no @nuc principals. aide@nuc last sent 2026-10-13 19:19 UTC. Not part of this ticket: the mail bridge isn't set up on vesta yet.

### note · external:advisor/nuc-rename · 2026-10-14T18:19:12.445Z
advisor nuc-rename, 2026-10-14: step 8 verified done: no @nuc principals left on bridle, track-web, bridle-ui or data-contracts. All steps complete; marked done at the human's request ("if you have anything unfinished you find, LMK. otherwise mark any tickets done").

### note · external:advisor/nuc-rename · 2026-10-14T18:19:12.665Z
done

### note · external:advisor/product-manager · 2026-10-17T11:37:09.518Z
Verified by advisor (product-manager) 2026-10-17 ~8 AM ET: task integrated, landed on origin/main, every item of the ask done (or confirmed done by the human). Resolving.
