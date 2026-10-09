+++
id = "br-hua2"
title = "Add a machine: one setup guide from bare OS to on the network (config, tokens, services, moving a project)"
kind = "chore"
state = "planned"
created_at = "2026-10-09T14:10:29.897Z"
updated_at = "2026-10-09T14:25:16.169803Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-09T14:10:40.375494Z"
ticket = "hua2"
+++

Ticket: docs/tickets/open/add-a-machine-one-setup-guide-from-bare-os-to-on-the-network-hua2.md (read it: the human's words and the five steps). Human priority 2026-10-09: used to bring the Windows PC onto the network. DOCS ONLY.

Write docs/context/add-a-machine.md: one procedure, in order, for any OS (macOS, Linux, WSL2), linking out for OS specifics instead of copying them: docs/context/windows-wsl2-host.md (stops at `just install`; add a link at its end "then continue at add-a-machine.md step 2"), docs/context/nuc-host.md (the move-a-project runbook), docs/context/adding-a-project.md, docs/design/cli.md (command reference), docs/design/agent-host/principals.md if present (grep for it; tokens).
Steps, each with the exact commands as they work TODAY (verify each against `bridle <cmd> --help` and the code; do not copy older doc text without checking; where a command does not exist yet say so and give the manual way):
1. GitHub and git: SSH key, `gh auth login` (for `[ci] github`), a distinct git identity per machine (ticket j7r4 rec 6, read it).
2. `~/.bridle/config.toml` on the new machine: `[machine] name`, `[machines]`, `[projects]`, workflow, budget, gateway, mail; and the matching lines to add on every other machine. Give a complete worked example for adding "<newpc>" to dalek and the NUC.
3. Tokens: (a) visitor tokens both ways, by hand (`bridle token create ...` flags per --help; they go in credentials.toml `[human.<machine>]`), noting `bridle token pair` (br-8c25, sk7p) will replace this; (b) peer tokens for mail: one per receiving project per sending machine, minted on the RECEIVER with `bridle token create --peer <SENDING machine> --project <receiving project>`, pasted under `[peer]` on the sender (ticket gdf3; br-gdf3 is fixing the CLI text, link to it as the rule); (c) a tokens checklist table: who mints what, where it is pasted.
4. Services: `bridle systemd install` (Linux/WSL2) or `bridle launchd install` (macOS), `bridle gateway install`, `bridle mail install` (check each exists and the flags), the workspace layout `<P>-workspace/<P>`, `loginctl enable-linger`, then `bridle doctor` to check, and what green looks like.
5. Moving a project onto it: link the nuc-host.md runbook and state the two traps: (i) remove the old machine's launchd plist / systemd unit first (note: `bridle systemd uninstall` is br-xrkh, not yet built; use `systemctl --user disable --now` meanwhile), (ii) take over by hand with `bridle serve --take-over` before enabling the new unit because a unit cannot take over. Note that automated project transfer is a later feature (do not describe it as available).
End with a 10-line "is it working" checklist (`bridle status --json` from the other machine, a cross-machine `bridle send`, a document read through the gateway).
Also add the page to the docs/README.md index.

Acceptance: ASCII only (rule ascii-in-editable-text); a reader can bring the Windows PC onto the network from this page alone; every command in it checked against --help or code and any unverifiable one marked "(unverified on a machine)". just check passes (docs only, run once). Model: Sonnet. Migration: none. Out of scope: any code, `token pair`, project-transfer automation, changing the WSL2 guide's content beyond the link.

## Thread

### note · external:advisor/product-manager · 2026-10-09T14:10:40.375Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-09T14:10:41.121Z
PdM (advisor product-manager): machine setup is the human's priority workstream (2026-10-09 ~10:15 AM ET: "I want that as a priority so that I can use it to set up the new Windows machine and add it to the network"; quoted in hua2/xrkh/kt25). Priority high; roadmap docs/notes/roadmap.md.
