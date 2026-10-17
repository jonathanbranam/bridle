---
id: br-r2nr
title: "Rename the NUC to vesta: host and bridle machine name, on both machines"
kind: chore
created: 2026-10-11
created_by: external:advisor/wsl2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-v7ug]
parent: br-du7e
theme: multi-machine
resolved: 2026-10-17T11:37:09Z
---

## The ask

The human, 2026-10-11 (to advisor wsl2): "Create a ticket for this claimed by human with the details. That assigns a todo to me". This replaces hf8a (removed: filed minutes earlier with the same steps as an ordinary ticket).


The human, 2026-10-11 (to advisor wsl2): "Can you also add instructions for renaming NUC plz"

The NUC's new name is `vesta` (the human, 2026-10-11: "let's go with Vesta for the NUC"; see
`docs/context/name-ideas.md` and `docs/context/naming.md`).

Then, follow-up messages: "Including for bridle" / "On both machines". So bridle's machine name
changes too, on dalek and on the NUC (step 6), not only the host name.

## Plan

Bridle knows the NUC by its machine name `nuc` in `~/.bridle/config.toml` (`[machine] name`,
`[machines]`, `[projects]`) and in `~/.bridle/credentials.toml`. `[machines]` maps a machine name
to the host name to reach it by. Separately, each NUC project's state-branch `owner.toml` and its
pre-push hook use the Linux host name (`hostname`), so changing the host name needs a take-over
per project (step 4).

The steps below are in the order to run them. The orchestrator checked the bridle parts against
the code on 2026-10-11 (its comment on this ticket's thread, "step 6 checked against the code",
has the reasons).

## Steps

0. **First, separately: upgrade the NUC to v0.6.2 with release self-upgrade**, and check it is
   healthy before going on (the human, 2026-10-11, via aide). The commands are aide's comment on
   this ticket's thread ("Step 0, before the rename"); they install into `~/.cargo/bin`, where
   the NUC's units run bridle from. The NUC also has a kernel reboot pending (7.0.0-38 installed,
   -34 running, checked 2026-10-11): reboot with the upgrade, not during the rename.

1. **Outboxes empty, both machines.** `bridle status` on dalek and the NUC; every `outbox` line
   must be gone or 0 queued. (Forward de-duplication is keyed by the origin machine, so a message
   retried across the rename could arrive twice.)

2. **On dalek: mint the new visitor tokens** before anything is renamed; the old ones keep working
   meanwhile. For each dalek project the NUC talks to, and each role the NUC has a
   `[<role>.dalek]` section for (orchestrator, advisor, aide, human):

   ```
   bridle --project <dalek-project> token create <role> --machine vesta --home <nuc-project>
   ```

   Paste each into the NUC's `~/.bridle/credentials.toml`, replacing the value under
   `[<role>.dalek]`. Don't use `token pair` (it mints visitors with no home, so replies don't
   come back). Optional: `bridle --project <dalek-project> token create --peer vesta` for the
   NUC's `[peer]` values (`/v1/forward` accepts any peer token whatever its name, so `peer:nuc`
   also keeps working).

3. **Host name and Tailscale name** (on the NUC):

   ```
   sudo hostnamectl set-hostname vesta
   sudo sed -i 's/^127\.0\.1\.1[[:space:]]\+nuc$/127.0.1.1 vesta/' /etc/hosts
   sudo tailscale set --hostname=vesta
   hostnamectl --static; grep 127.0.1.1 /etc/hosts; tailscale status --self   # all say vesta
   ```

   If the Tailscale admin console still shows `nuc`, it was renamed there by hand (that pins it):
   rename it in the console too (Machines, the machine's menu, Edit machine name). The name `nuc`
   stops resolving at once, so go straight on.

4. **On the NUC, per project: take ownership again under the new host name.** Without this each
   NUC daemon refuses to start at its next restart (exit 78, which the systemd unit treats as
   final), and the hook refuses pushes to the integration branch. For each of meta-notes, notes,
   dotfiles-local, meta-notes-ui:

   ```
   systemctl --user stop bridle-<p>.service
   cd <p's clone>                                  # serve uses the cwd's project, not --project
   bridle serve --take-over                        # runs the daemon; leave it running
   bridle status --project <p>                     # in a second shell: wait for the state line to say pushed
   # then Ctrl-C the serve, and:
   systemctl --user start bridle-<p>.service
   ```

   `serve --take-over` runs the daemon until stopped; the new `owner.toml` is written at its next
   state flush, so stop it only once `bridle status` shows the state as pushed (orchestrator,
   from docs/design/cli.md "Owner refusal exit code").

5. **Config, both machines, in one go.** No daemon restart is needed for this (routing and the
   sender's machine label are read from config per send), except the NUC's gateway.

   - dalek `~/.bridle/config.toml`: in `[machines]`, `nuc = "nuc"` -> `vesta = "vesta"`; every
     `[projects]` entry `machine = "nuc"` -> `machine = "vesta"`.
   - dalek `~/.bridle/credentials.toml`, in the same edit: rename the sections `[advisor.nuc]`,
     `[human.nuc]`, `[orchestrator.nuc]` to `.vesta` (the CLI looks them up by machine name; the
     tokens inside don't change).
   - NUC `~/.bridle/config.toml`: `[machine] name = "vesta"`; the same `[machines]` and
     `[projects]` edits as dalek; `[gateway] public_url = "http://vesta.tailbc91f5.ts.net:7878"`.
     Then restart the NUC's gateway.
   - remus, once bridle is installed there (ticket v7ug): the same `[machines]` and `[projects]`.

6. **SSH and dotfiles.** `ssh vesta` works through MagicDNS; `ssh-keygen -R nuc` on dalek and remus
   clears the old host key (Tailscale SSH, so nothing to copy). Fix anything that says `ssh nuc`.
   dotfiles-local doesn't break (`host-nuc/rcrc` sets `HOSTNAME="nuc"` itself); renaming
   `host-nuc/` to `host-vesta/` is optional and can come later (`RCRC=<clone>/host-vesta/rcrc rcup
   -B vesta` re-links `~/.rcrc`).

7. **Check.** `bridle doctor` and `bridle daemons` on each machine. A message each way arrives
   (dalek -> a NUC project, and the NUC's orchestrator -> bridle) and its `from` says `@vesta`.
   The gateway opens at the new URL.

8. **Days later: revoke the old `@nuc` tokens**, on each dalek project, once the old threads are
   done (a reply to an old `@nuc` message routes only while that principal exists; afterwards
   it fails "no such recipient"):

   ```
   bridle --project <p> token revoke orchestrator@nuc   # and human@nuc, aide@nuc, advisor@nuc, peer:nuc
   ```

No change needed: systemd unit names (`bridle-<project>.service`, keyed by project; don't re-run
`bridle systemd install`), the daemon registry, remote watchers, and the `@dalek` visitors on the
NUC. History (stored messages and events saying `@nuc`, each daemon's principals rows, the docs
and incident history) stays as it is.

