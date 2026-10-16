+++
id = "br-7bs2"
title = "macOS firewall still blocks a freshly launched bridle (gateway) after a self-upgrade, despite the stable signing identity (p88z)"
kind = "bug"
state = "planned"
created_at = "2026-10-15T23:15:13.826Z"
updated_at = "2026-10-16T01:30:02.982900Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
priority_at = "2026-10-15T23:17:44.940888Z"
ticket = "7bs2"
+++

docs/tickets/open/macos-firewall-still-blocks-a-freshly-launched-bridle-gatewa-7bs2.md

## Thread

### note · external:aide · 2026-10-15T23:17:29.636Z
The human, 2026-10-15 ~7:20 PM ET, verbatim: "Pretty much every time I open the laptop there is another Allow dialog showing. I've clicked it a dozen times in the past week". So the Allow does not stick under the 'bridle local signing' identity either: the 'click Allow once more' workaround is ruled out. Recommended direction (aide): stop bridle being an inbound listener the firewall sees (127.0.0.1 + tailscale serve), plus the post-restart reachability check.

### note · external:aide · 2026-10-15T23:17:29.678Z
priority: normal -> high

### note · external:aide · 2026-10-15T23:17:34.243Z
priority: high -> normal

### note · external:advisor/product-manager · 2026-10-15T23:17:44.940Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-15T23:17:45.119Z
advisor product-manager: priority high (PdM call): the human hits the Allow dialog on most laptop opens, and a missed one cuts the gateway off from the phone and the other machines.

### note · external:aide · 2026-10-15T23:18:47.380Z
The human, 2026-10-15 ~7:20 PM ET, verbatim, on the aide's recommended fix (bridle listens on 127.0.0.1, fronted by tailscale serve): "I don't like that solution." Don't build that direction.

### note · external:aide · 2026-10-15T23:20:04.754Z
The human's decision, 2026-10-15 ~7:25 PM ET, verbatim: "(1) for now. 2 optional. I agree not 3. If we can't fix it I'd rather use Tailscale than disable the firewall". Options as put to them by the aide: (1) find why the firewall ignores the 'bridle local signing' signature and fix that (e.g. a self-upgrade replaces the file at the path, so the firewall entry may be tied to the old file; rewriting in place might fix it), investigate on dalek first; (2) optional: re-add bridle to the firewall after each upgrade with socketfilterfw --add via a one-time NOPASSWD sudoers entry; (3) rejected: disable the macOS firewall. Fallback if (1) can't be fixed: 127.0.0.1 + tailscale serve (preferred over disabling the firewall). This supersedes the earlier 'don't build that direction' note: Tailscale is the fallback, not ruled out.

### note · external:advisor/product-manager · 2026-10-15T23:20:14.659Z
Readied by advisor product-manager on the human's direction via aide (m-13700): "(1) for now. 2 optional. I agree not 3. If we can't fix it I'd rather use Tailscale than disable the firewall". Build option 1 (find why the firewall ignores the signing identity on dalek; fix the signing/upgrade path); option 2 optional; never disable the firewall; tailscale serve is the fallback.

### note · agent:pm-1 · 2026-10-15T23:20:59.346Z
Brief (pm-1). Model: Sonnet. Size: M, investigation first. High priority (the human hits Allow on most laptop opens; a missed one cuts the gateway off from the phone and other machines). Human's decision: option 1 now (find why the firewall ignores the "bridle local signing" identity, fix the signing/upgrade path); option 2 optional; NEVER disable the firewall; 127.0.0.1 + tailscale serve is the fallback only if 1 cannot be fixed. Do not build the fallback without telling the manager first.
Step 1 (report findings on the thread before building): on dalek (this machine, macOS), reproduce with a controlled experiment, not guesses: which path does the firewall list (`/usr/libexec/ApplicationFirewall/socketfilterfw --listapps`) hold for bridle, and does the entry survive a replaced file? Test hypotheses: (a) the self-upgrade/`bridle sign binary` replaces the file at ~/.cargo/bin/bridle (new inode) and the firewall entry is tied to the old file/cdhash, so rewriting the signed binary IN PLACE (same inode) or signing before the final rename may keep it; (b) the entry is keyed on cdhash not the designated requirement for a self-signed leaf; (c) a plain launchd restart (same file, no re-sign) does not re-prompt, which isolates the signing step. Use a throwaway copy of a small signed binary listening on a spare port for the experiments, not the live gateway; avoid prompting the human (note any dialog that appears). Read crates/bridle sign code (git grep -n "bridle local signing\|codesign" crates/ justfile), docs/tickets/resolved for p88z, and the self-upgrade install path (crates/bridle-daemon upgrade/install).
Step 2: fix the cause in the install/sign path so an upgrade leaves the firewall decision intact; document in CLAUDE.md "Build configuration"/docs/design/agent-host as the finding dictates, plus CHANGELOG.
Step 3 (only if cheap, optional per the human): after a restart the gateway/daemon self-checks reachability on its tailnet address and raises a notice/incident naming the macOS firewall as the likely cause, so the human is not the detector.
Stop points: land step 1's findings (a docs note on the ticket thread) and step 2 if found; if no cause is found, say what was ruled out and stop, do not guess a fix. Option 2 (sudoers + socketfilterfw --add) is out of scope unless the manager asks: it needs the human's sudoers edit. Acceptance: just check; for the fix, verified across a real self-upgrade or a simulated binary replace on dalek, stated in the done note.

### note · external:aide · 2026-10-15T23:21:47.572Z
aide, 2026-10-15 ~7:25 PM ET: the human clicked Allow on dalek; from remus, http://dalek.tailbc91f5.ts.net:7878/ now answers 200 in 12 ms (was timing out). Gateway pid 88900 was not restarted, so the Allow took effect on the running process.

### note · external:advisor/product-manager · 2026-10-15T23:47:34.193Z
From the human, via advisor product-manager (2026-10-15 ~7:35 PM ET): "7bs2 might need internet websearch to properly diagnose". So spawn step 1 (the diagnosis) as the researcher role (worker + WebSearch/WebFetch, config.rs researcher_default), not a plain worker, which has no web tools. Cite sources (Apple docs, forums on socketfilterfw / application firewall and code signing identity).

### note · external:advisor/product-manager · 2026-10-15T23:47:38.739Z
Correction: that note was ~7:47 PM ET, not ~7:35.

### note · agent:pm-1 · 2026-10-15T23:47:40.791Z
Brief amendment (pm-1, for the manager): step 1 (the diagnosis) is run by the researcher role (`bridle agent spawn researcher --ticket br-7bs2 ...`), per the human via advisor product-manager, because it needs web search. Researcher: besides the controlled experiments, search Apple documentation and forums (socketfilterfw / Application Firewall, how its allow list identifies an app: path, cdhash, or designated requirement; behaviour with self-signed certificates and with a file replaced at the same path), and include the c2 wiki per rule c2-wiki-in-searches if relevant. Cite sources, write findings and a recommended fix on this thread, then stop. Step 2 (the fix) is a separate worker spawn by the manager after the findings are read, using the findings.

### note · agent:7bs2 · 2026-10-15T23:54:11.759Z
# br-7bs2 step 1: diagnosis findings (researcher 7bs2, dalek, macOS 14.8.3 x86_64)

Status: cause NOT proven. Strong leads below. The one experiment that would prove it
(a real listener from a throwaway signed copy) cannot run without (a) a firewall dialog on
dalek's screen and (b) root to remove the throwaway entry afterwards (`sudo -n` needs a password).
I did not run it. Decision needed from manager-2: see "What I need".

Note: `bridle ticket claim br-7bs2` failed: "conflict: task br-7bs2 is settling until 7:57 PM".
I did read-only diagnosis without the claim.

## Facts measured (all passive, no dialog, live gateway untouched)

1. The allow list is /Library/Preferences/com.apple.alf.plist, key `applications` (readable
   without root with `plutil -p` / python plistlib). Each entry has: `alias` (a classic Alias
   record: volume name+creation date, **file name, file creation date**, POSIX path), `reqdata`
   (a compiled code requirement blob), `state`, and sometimes `bundleid`.
2. Decoding `reqdata` with `csreq -r- -t`:
   - bridle (entry 10): `identifier bridle and certificate leaf = H"37e545f6...e6c3"` = the
     designated requirement of the current binary and the current "bridle local signing" leaf.
   - ad-hoc binaries (python3.10 pyenv, caddy 2.11.2, caddy 2.11.4): `cdhash H"..."`, one entry
     per build and per path. caddy has two entries (Cellar/2.11.2 and 2.11.4): a new build means a
     new entry and a new prompt. This is the pre-p88z state for bridle, and still the state for
     any other ad-hoc listener.
   So (b) "keyed on cdhash instead of the DR for a self-signed leaf" is RULED OUT as the stored
   form: for bridle the stored requirement is the DR with the leaf hash, as p88z intended.
3. The bridle entry's alias records the file creation date 2026-10-04 17:09:20. The current
   ~/.cargo/bin/bridle was born 2026-10-15 18:41:36 (new inode 144045688, the self-upgrade
   replaced the file). For caddy 2.11.4 the alias creation date equals the file's birth
   (Jun 1 15:35:02), i.e. entries do record the file's birth time.
   The plist mtime is Oct 12 12:39: **the human's Allow click at ~19:25 on Oct 15 did not rewrite
   the plist** (entry still names the Oct 4 file), yet the click un-blocked pid 88900. So the
   decision that blocked the new gateway was not "no entry for this path": an entry for this
   exact path with a matching DR already existed.
4. `socketfilterfw --getappblocked ~/.cargo/bin/bridle` (works without root) says "permitted"
   even while the new gateway was blocked (the aide saw this too). So the query is a path lookup
   and is NOT evidence of what is enforced for a running process.
5. The lookup is by path, not by DR: byte-identical copies of the signed binary (cp, cp -c
   clone) at /tmp/7bs2/x/ with the same DR report "The application is not part of the firewall".
   A DR match alone does not carry an allow to another path.
6. `allowsignedenabled`/`allowdownloadsignedenabled` are both 1. A Developer-ID node
   (`~/.nvm/.../node`, Authority Apple Root CA chain) listens on `*:6010...` with NO list entry
   (`--getappblocked`: "not part of the firewall") and is not blocked: signed software is
   auto-allowed. Bridle's self-signed cert never qualifies for that route, so bridle depends on
   the per-path list entry alone. (Inference: the auto-allow path needs a chain to an Apple/trusted
   root, not a user-domain codeSign trust setting; the cert is user-trusted for codeSign
   (`security dump-trust-settings` Cert 2) and still gets no auto-allow, as the blocked gateway shows.)
7. Four bridle processes listen on the tailnet address 100.100.189.100: ports 7401, 7405,
   7406, 7878. All run /Users/jbranam/.cargo/bin/bridle, so one list entry has to cover all.
8. socketfilterfw (pid 265) log (`/usr/bin/log`, note zsh's `log` builtin shadows it):
   -67065 (errSecCSNoSuchCode) at 18:41:48, 18:41:49, 18:41:55, 18:42:05, around the gateway's
   18:41:57 start, and many -67062 (errSecCSUnsigned) per minute all day (those are unsigned/other
   processes; ~1 pair every few seconds). Too sparse to attribute to bridle. No appfirewall.log
   (/var/log/appfirewall.log is 0 bytes).

## Reading

The DR form is right, so the "sign with a stable identity" part of p88z did what it intended
(reqdata). What fails is the other half of the match. Leading hypothesis (H1, unproven): the
firewall also ties the entry to the file object recorded in the Alias (file creation date /
file identity) or to the path-at-launch resolution, so a self-upgrade that **replaces** the file
(new inode, new birth time) makes the running-process check fail even though path and DR still
match; the click then grants it for that process only (no plist write). That fits: the human
clicks Allow "a dozen times in a week" (every upgrade/replace), while the plist stays old.
Alternative (H2): the dialog the human sees on laptop open is for a *different* bridle binary
(e.g. a `target/debug/bridle` from agent worktrees running `bridle dev site up`, which is
ad-hoc, new path/cdhash every build: exactly the pre-p88z behaviour). The dialog says only
"bridle", so the human cannot tell. I did not see any such listener now (no non-installed bridle
listening), so H2 is unverified, not ruled out.

## Ruled out
- (b) cdhash-keyed entry for bridle: ruled out (reqdata is the leaf-hash DR).
- Identity missing/untrusted at sign time: ruled out (`codesign -dvvv` Authority=bridle local
  signing, 1 valid identity, user codeSign trust present).
- "Entry absent for the path": ruled out (entry exists, `--getappblocked` permitted).
- DR alone carries the allow across paths: ruled out (item 5).
- Whether a launchd restart with NO re-sign (hypothesis c) re-prompts: NOT tested (needs a live
  listener; see below). Whether in-place rewrite (a) keeps the decision: NOT tested.

## Web sources (what they do and don't say)
- Apple Developer Forums 740747, https://developer.apple.com/forums/thread/740747 : a binary
  re-prompted every launch although `socketfilterfw --listapps` showed the choice; fixed after
  the binary was ad-hoc signed (signing had failed because the containing directory was
  read-only to the user). It contains NO Apple-staff statement on how the firewall identifies
  programs. (My first search summary attributed a "firewall uses the DR" quote to Apple DTS
  (Quinn); the fetched page does not contain it. Treat that attribution as unverified.)
- Jamf / krypted threads on `socketfilterfw --add`, `--setallowsigned`, plist location
  (https://krypted.com/mac-security/command-line-firewall-management-in-os-x-10-10/embed ,
  https://community.jamf.com/t5/jamf-pro/firewall-issue-on-10-9-3-and-above/td-p/35334 ,
  https://lists.macports.org/pipermail/macports-dev/2022-March/044167.html): confirm the CLI and
  that unsigned binaries behave inconsistently; nothing on self-signed leaf vs replaced file.
- No Apple document found that states how the firewall matches (path, cdhash, DR). The
  decoded plist (item 1-3) is the best primary evidence.
- c2 wiki: tried https://c2.com/wiki/remodel/pages/CodeSigning -> HTTP 404; the topic (macOS
  firewall internals) is not a c2 topic, nothing relevant found.
- Failed/odd: WebFetch of the Apple forum thread returned only two posts, both by the poster.

## What I need (proposed experiment, needs your call)
Enforcement test with a throwaway: copy a small signed binary (e.g. tiny listener) to
/tmp/7bs2/fw/t, sign with the identity, listen on 0.0.0.0:<spare>, connect from remus over
tailscale. Steps: (1) first listen -> a dialog appears on dalek (needs a click; I can try
osascript to click Allow on the throwaway, may need Accessibility permission); (2) restart same
file (c); (3) re-sign in place with `codesign -f` (a); (4) replace via cp+mv (new inode, same
bytes) (H1); (5) replace with different bytes re-signed. Reading the plist after each tells
whether the Allow is stored. Cost: one stale throwaway entry that only root can remove
(`sudo socketfilterfw --remove /tmp/7bs2/fw/t`), which the human or a sudoers entry would have
to do. Alternatively, run the same tests against the real path ~/.cargo/bin/bridle at a quiet
moment with a spare-port listener copy... not recommended.
Cheaper evidence the human can give: next dialog, screenshot/the path shown in "Show details"
(distinguishes H1 from H2).

## Recommended fix directions (not built)
- If H1: make install not leave a new file object: write the new signed bytes into the
  EXISTING inode (truncate+write in place, or `codesign -f` in place), though the running
  binary's text pages make in-place writes risky; test first. Alternatively `socketfilterfw
  --remove` + `--add` + `--unblockapp` after each install (needs root = option 2).
- If H2: stop agent/dev-site listeners from binding non-loopback addresses (bind 127.0.0.1 in
  dev sites), so only the installed binary can ever prompt.
- Detection (step 3) stays worthwhile whatever the cause.

### note · agent:7bs2 · 2026-10-15T23:54:11.826Z
step 1 findings posted on the ticket. Cause not proven: bridle's list entry is the correct leaf-hash DR (cdhash theory ruled out) but the alias records the old file's birth time (Oct 4) and the Allow click wrote nothing to the plist; leads H1 (replaced file) vs H2 (dev-site/worktree bridle listeners). Proving it needs a throwaway listener -> a dialog on dalek plus root to remove the entry; need your go-ahead. Claim failed: 'settling until 7:57 PM'.

### note · external:aide · 2026-10-15T23:59:36.708Z
The human, 2026-10-15 ~8:00 PM ET, verbatim, on the findings and the throwaway-listener question (m-13743): "I don't udnerstand this though - the bridle gateway was clearly down and refusing connections. this isn't the solution. Sure, a worker may run an unsignged bridle but that doesn't explain why the actual bridle gateway on 7878 was not responding until I clicked Allow. Unless it is being replaced by a worker process." aide: confirmed the blocked process was the installed gateway (pid 88900, ppid 1 = launchd, /Users/jbranam/.cargo/bin/bridle gateway, started 18:41:57 after the self-upgrade replaced the file at 18:41:36), not a worker's binary, and the human's Allow unblocked that pid. So H2 can't explain the outage; the observed facts already fit H1 (a self-upgrade that replaces the file makes the next freshly launched installed bridle be blocked, despite a matching path and DR).

### note · external:aide · 2026-10-16T00:00:56.603Z
From the human, via aide, answering m-13743 (br-7bs2), verbatim: "I am happy to test a SIGNED copy of bridle for the firewall. Also - is it possible we need to remove bridle from the firewall first to \"refresh\" it?" Earlier, verbatim: "the bridle gateway was clearly down and refusing connections ... Sure, a worker may run an unsignged bridle but that doesn't explain why the actual bridle gateway on 7878 was not responding until I clicked Allow." So: go ahead with the throwaway test using a copy signed with "bridle local signing" (the researcher has the researcher ask the human first, at dalek). Please also test the human's refresh idea: the ~/.cargo/bin/bridle entry's alias still records the Oct 4 file and the Allow click didn't rewrite the plist, so a sudo socketfilterfw --remove + --add (+ --unblockapp) of ~/.cargo/bin/bridle may fix it; the next self-upgrade then shows whether a replace breaks it again (H1) or the stale entry was the whole problem. Both need the human at dalek for sudo; ask them via aide.

### note · agent:manager-2 · 2026-10-16T00:01:16.934Z
From the human via orchestrator (m-13759): go ahead with the throwaway listener test, using a copy signed with 'bridle local signing' (not ad-hoc). Also test the refresh idea: sudo socketfilterfw --remove then --add (+ --unblockapp) of ~/.cargo/bin/bridle (its alias still records the Oct 4 file); the next self-upgrade then shows whether a file replace breaks it again (H1). Note: the real gateway on 7878 refused connections until Allow was clicked, which H2 (worker/dev-site listeners) does not explain. Both steps need sudo at dalek: ask the human via external:aide before running each. Record results here.

### note · agent:7bs2 · 2026-10-16T00:01:31.881Z
7bs2 (researcher) needs the human at dalek for the firewall test. Throwaway: /tmp/7bs2/fw/t, signed 'bridle local signing', same designated requirement as the real bridle. Ask 1: when I say go, a firewall dialog will appear for 't' (path /tmp/7bs2/fw/t); please click Allow. Ask 2 (later, sudo): socketfilterfw --remove then --add (+--unblockapp) for /tmp/7bs2/fw/t, then for ~/.cargo/bin/bridle. Reply with 'go' when the human is at dalek and ready for the dialog.

### note · agent:7bs2 · 2026-10-16T00:39:14.833Z
7bs2: reproduced a block on the throwaway (listener /tmp/7bs2/fw/t, port 60758 stays unreachable from remus though --getappblocked says permitted, same as the real outage). Please ask the human: is a firewall dialog showing on dalek right now for 't'? If yes, what exactly does it say (app name/path), and please do NOT click yet; reply with the text. Then click Allow and tell me.

### note · agent:7bs2 · 2026-10-16T00:57:23.860Z
# br-7bs2 interim results (researcher 7bs2), experiments on throwaway /tmp/7bs2/fw/t

Setup: tiny C listener on 0.0.0.0:60758, signed `codesign -s "bridle local signing" -i bridle` (DR identical to the real bridle's: `identifier bridle and certificate leaf = H"37e5..."`). Probed from remus via `ssh remus curl http://100.100.189.100:60758/` (200 = reachable, 000 = blocked). Shell-started and launchd-started (gui domain) both tried.

| step | result |
|---|---|
| first listen | dialog; human clicked Allow; plist gained NEW entry 13 (same DR as entry 10, own alias for /private/tmp/7bs2/fw/t). So Allow does write the plist. Entries are per path. |
| running after Allow | 200 |
| restart same file, no re-sign (hypothesis c) | 200 |
| `codesign -f` re-sign of the file (a) | 200 (this changes the inode too) |
| `cp t t.new && mv t.new t` (new inode, birth, same bytes) (H1) | 200 |
| new bytes, new cdhash, signed with -i bridle, mv over | 200, no plist change |
| launchd-started instead of shell child | 200 |
| replace with a fresh UNSIGNED cc build, `codesign --force -s ... t` (no -i) then `launchctl kickstart -k` | **000 blocked**. DR became `identifier t` (force-sign takes the identifier from the file name), differing from the entry's `identifier bridle`, so this proves the DR (identifier+leaf) IS enforced, not just path. |
| then re-signed with `-i bridle` (DR back to the entry's), kickstart | **still 000** although `--getappblocked` says permitted and lsof shows the process listening. Same signature of failure as the real outage. |

Findings so far
- H1 (file replaced: new inode/birth/cdhash) does NOT by itself break an allowed entry: three replace variants stayed reachable with no new dialog and no plist change.
- Hypothesis (c) restart w/o re-sign: fine. Hypothesis (a) in-place re-sign: fine.
- A launch whose code requirement did NOT match the entry (identifier changed) is blocked; and the block persisted after the requirement matched again, on a fresh process. So a state "blocked/pending" seems to stick per path after one non-matching launch, until a dialog is answered (the real gateway: same, blocked until the human clicked Allow). `--getappblocked` is blind to it.
- So the live question is what made the real gateway's launch at 18:41:57 not match. Candidates, now narrowed: the identifier. `codesign --force -s name <exe>` (crates/bridle-daemon/src/signing.rs:69) sets the identifier from the file name; cargo's linker ad-hoc signature uses `bridle-<hash>` (seen in .bridle/upgrade-target/release/bridle: Identifier=bridle-8a9cb0b0467401f1), so any launch of a binary that is still linker-signed (not yet re-signed), or a copy/rename under another file name (identifier = that name), carries a different identifier and will not match the `identifier bridle` entry. E.g. `bridle.prev`, or the upgrade's self-check run of the new build from upgrade-target (named bridle, ad hoc `bridle-<hash>`), if it ever opens a listener. NOT yet shown for the real outage: the installed file at 18:41:57 was signed (`codesign -dvvv`: Identifier=bridle).
- Still pending: the text of the pending dialog (human away until ~21:10 ET); I left the blocked throwaway listener running (launchd job dev.bridle.7bs2test, port 60758) for that. Cleanup will need the human's sudo: `socketfilterfw --remove /private/tmp/7bs2/fw/t` (plus the stale `/tmp/7bs2/fw/t` entry if any).
- The "remove then add bridle" refresh test (needs sudo) is not run yet.

### note · agent:7bs2 · 2026-10-16T01:30:02.982Z
7bs2: yes, please have the human click Allow on that 't' dialog now. I'll check the plist and reachability right after.
