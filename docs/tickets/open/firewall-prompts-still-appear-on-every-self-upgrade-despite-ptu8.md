---
id: br-ptu8
title: "Firewall prompts still appear on every self-upgrade despite the stable signing identity: find the break in the signing chain"
kind: bug
created: 2026-11-14
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

Filed by aide. Follows br-qy4y (resolved 11-11 with "live with it" plus a warning; cause never proven), br-7bs2, br-p88z.

The human, 2026-11-14 (to aide), verbatim: "I don't understand why the signing fix is not working; i suspect there's a bug or issue somewhere. a loss in the signing chain after a reboot or a timing issue? we had a timing issue before where it wasn't signed before the restart happened, or something else. it's very irritating and hard for me to believe this is normal and expected behavior on a mac"

## What happened today (dalek, 2026-11-14)

The human got several batches of "accept incoming connections" prompts for bridle: around 12:31Z (gateway restart), 12:41Z (bridle-ui, data-contracts, track-web daemon restarts for br-z76j) and 12:50Z (bridle daemon self-upgrade to 0.8.0, which relaunched the launchd jobs).

Evidence gathered by aide, 12:55Z:
- daemon.log 12:50:27Z: `installed binary signature before restart exe=~/.cargo/bin/bridle identifier="bridle" requirement=identifier bridle and certificate leaf = H"37e545f6..."`, then 12:50:27.9Z `restarted launchd jobs onto the upgraded binary` (6 jobs). Same requirement as every upgrade since 11-09.
- `codesign -dvvv ~/.cargo/bin/bridle`: Identifier=bridle, Authority=bridle local signing, Signed Time 12:41:25Z, CDHash ef3d0ee3... The file was installed at 12:41:25Z (stat mtime; inode 194061122): so an install happened at 12:41, not only at 12:50. Which installer ran then (another project's daemon self-upgrading? the orchestrator's restarts?) and whether the 12:50 upgrade re-installed or found it current is worth checking.
- daemon.log 12:42:34Z: incident "The gateway cannot connect to its own address 100.100.189.100:7878 after a start", i.e. a listener was blocked until the human answered a dialog.
- `socketfilterfw --listapps`: one entry, ~/.cargo/bin/bridle, "Allow incoming connections".

So the signing step logs success, before the restart, with an unchanged designated requirement, and the firewall still re-asks each newly launched process.

## The ask

Find the actual cause and fix it so a self-upgrade on dalek shows no firewall prompts. The human doesn't accept "normal on a Mac" without proof. Candidates, including the human's:
1. A timing or ordering gap: a process launched (or relaunched by launchd) before the signed copy is renamed into place, or something other than `signing::install` writing ~/.cargo/bin/bridle (several daemons on dalek self-upgrade; check every install path, including `just install` and each project daemon's upgrade, signs and renames atomically).
2. A loss after reboot (prompts started after the 11-08 reboot): keychain or trust state for the self-signed "bridle local signing" leaf, or kernel ALF state.
3. qy4y's lead (a): the firewall's entry keyed on the file (alias / inode) rather than the requirement, so every rename-into-place is a new app to it; in-place overwrite would then be the fix.
4. Self-signed leaf not trusted for code signing, so ALF falls back to the cdhash.

Use qy4y's cheap experiment (alf.plist entry with `plutil -p`, `stat` of the binary, `log show --predicate 'process == "socketfilterfw"'` before and after one upgrade). A test that raises a dialog needs the human at dalek; ask through the aide. Don't switch bridle to localhost plus `tailscale serve` without the human's go (that was option B on qy4y).
