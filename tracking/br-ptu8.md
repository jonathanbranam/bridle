+++
id = "br-ptu8"
title = "Firewall prompts still appear on every self-upgrade despite the stable signing identity: find the break in the signing chain"
kind = "bug"
state = "claimed"
created_at = "2026-11-14T12:57:20.899Z"
updated_at = "2026-11-14T15:11:56.288386Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:ptu8",
]
summary = "Research only, nothing merged. Apple (TN2206, Quinn DTS 747298) documents the firewall tracking apps by designated requirement and not trusting self-signed signers for its anchor check; dalek's leaf is untrusted (CSSMERR_TP_NOT_TRUSTED, no trust settings). alf.plist holds bridle's stable DR entry yet first launches of a replaced file prompt (12:50 batch, 3 dialogs = 3 listeners); same-file restart did not (12:41 ui). Timing/sleep hypotheses rejected. Cause unproven: ranked candidates, three experiments and a proposed fix posted on the thread, awaiting the human."
ticket = "ptu8"
+++

docs/tickets/open/firewall-prompts-still-appear-on-every-self-upgrade-despite-ptu8.md

## Thread

### note · external:aide · 2026-11-14T12:57:28.050Z
readied by external:aide

### note · external:aide · 2026-11-14T12:57:28.160Z
Readied by aide: a bug with a clear ask (rule ready-without-asking); the human asked for the cause to be found (quote above).

### note · agent:pm-1 · 2026-11-14T12:58:07.973Z
pm-1 brief: model Sonnet. This is diagnose-then-fix, in that order, on one branch.
1. Diagnose first, on the ticket thread, before changing code: read docs/tickets/resolved/ for qy4y, 7bs2 and p88z and crates/bridle/src/signing.rs. List every code path that writes ~/.cargo/bin/bridle (signing::install, the daemon self-upgrade, just install, `bridle sign binary`) and say for each whether it signs a copy named `bridle` and renames it into place; the 12:41:25Z install the aide found must be explained (which installer ran, did the 12:50Z upgrade re-install). Check candidates 1 and 4 from the ask from code and logs (the worker can run codesign, stat, plutil on the dalek machine it runs on).
2. Fix what the code shows (for example an install path that does not sign-and-rename, or a launchd restart before the rename). If the cause is the OS keying the firewall entry on file identity (candidate 3) or needs a dialog to reproduce, stop and post findings; the aide gets the human to run one upgrade at dalek, and the fix follows on this ticket or a slice.
3. Do not switch to localhost plus tailscale serve (human's go needed). Candidate 2 (reboot state) is a human-at-dalek check, not the worker's.
Acceptance: findings on the thread; any code fix has a test of the install ordering (signed before the restart, atomic rename); `just check` with BRIDLE_* env unset. Docs: the signing section of CLAUDE.md or docs/design where the chain is described, if behaviour changes.

### note · external:orchestrator · 2026-11-14T12:58:58.081Z
orchestrator: the 12:41:25Z installer was the bridle daemon's own self-upgrade to 8352cc3, not my daemon restarts. Evidence:
- bridle/.bridle/upgrade-root/bin/bridle mtime 12:41:24Z.
- ~/.cargo/bin/bridle born 12:41:24Z (new inode 194061122), signed 12:41:25Z, Authority=bridle local signing, Identifier=bridle, CDHash ef3d0ee3.
- run_build() in crates/bridle-daemon/src/upgrade.rs calls signing::install (copy, codesign, rename over the live path) as soon as the build finishes. It does not wait for the drain. The bridle daemon then drained on worker nxkj and only restarted at 12:50:27Z.
- The gateway saw the changed binary at 12:41:34Z ('the bridle binary changed; restarting onto it'), re-exec'd onto the new inode, and at 12:42:34Z raised 'cannot connect to its own address 100.100.189.100:7878'. So the dialog appeared within a minute of the install.
- My restarts of bridle-ui, data-contracts and track-web (12:41:21-26Z) only logged the signature ('installed binary signature before restart'; log_signature only reads). They wrote nothing.
- Same sequence as before: a new inode at the same path, the same identifier, and the same certificate leaf 37e545f6. The firewall still asked again. That fits br-7bs2's finding that the decision is keyed by path plus something the rename changes (cdhash?), not the designated requirement alone.
- Side note: 'the clock jumped: the machine slept' at 12:31:08Z in the bridle daemon log. That is relevant to the human's reboot/timing suspicion.

### note · external:aide · 2026-11-14T13:12:36.799Z
The human, 2026-11-14 (to aide), verbatim: "hmm... yeah let's verify before that; it sounds risky. Can we do some more internet research on the os x behavior? It should be clearly documented. I don't understand why we are guessing here. Do i need to find and download some docs from apples developer resource center?"

Change to the brief (aide, for the human):
1. FIRST, research, by a worker with the `researcher` role (web tools): what Apple documents about how the macOS application firewall (socketfilterfw / ALF) identifies an allowed app and when it re-prompts: code signature vs designated requirement vs cdhash vs path/inode; self-signed certificates; behaviour after a binary is replaced or updated in place; after reboot. Cite sources (Apple support and developer docs, Platform Security Guide, man socketfilterfw, Apple DTS forum answers by Apple staff such as Quinn), and say plainly where Apple documents nothing and what is only folklore.
2. Then verify the cause on dalek with evidence, as the brief already says.
3. NO change to how bridle installs (no in-place overwrite, no tailscale serve) until the cause is verified AND the human has approved the fix. The human called the in-place overwrite "risky". Post findings and the proposed fix on this thread; the aide brings them to the human.

### note · agent:pm-1 · 2026-11-14T13:13:00.617Z
pm-1 REVISED brief (supersedes my 12:58Z brief; follows the human's change above and the orchestrator's note). This ticket is now research and verification ONLY. No code change, no install-path change.

Worker: role `researcher` (web tools); model Sonnet. Ask manager-2 to spawn it with that role; if it cannot, say so on this thread.

1. Research. What Apple documents about how the macOS application firewall (socketfilterfw / ALF) identifies an allowed app and when it re-prompts: code signature vs designated requirement vs cdhash vs path or inode; self-signed certificates; a binary replaced by rename-into-place vs overwritten in place; behaviour after reboot or sleep. Sources: Apple support and developer docs, the Platform Security Guide, `man socketfilterfw`, Apple DTS forum answers by Apple staff (Quinn). Include the c2 wiki per rule c2-wiki-in-searches if it has a relevant page (say if unreachable). Say plainly where Apple documents nothing and what is only folklore. Read first: resolved tickets qy4y, 7bs2 and p88z, and the orchestrator's evidence note above (signing::install runs as soon as the build finishes, before the drain; the dialog appeared within a minute of that install, before the 12:50Z restart; the daemon log shows a sleep clock jump at 12:31Z).
2. Verify on dalek with evidence, using only read-only commands (codesign, stat, plutil -p on alf.plist, log show for socketfilterfw, listapps). Any experiment that raises a dialog goes through the aide, since the human is at dalek today; the worker proposes the exact steps and waits.
3. Post findings on this thread: the cause (proven, or ranked candidates with what each test showed), citations, and a PROPOSED fix with its risks (for example in-place overwrite, which the human called risky). The aide takes it to the human. A fix is a new ticket after the human approves; this ticket resolves when the cause is established or the research is shown inconclusive.

Out of scope: any change to signing.rs, upgrade.rs, install or launchd handling; tailscale serve. Acceptance: findings on the thread with citations, nothing merged.

### note · agent:ptu8 · 2026-11-14T15:11:29.226Z
researcher ptu8 FINDINGS (read-only; no firewall, signing, install or launchd change; nothing merged). Dalek, macOS 14.8.3 (23J220), x86_64, booted 11-08 08:46 local. Times: local = UTC-5, so 07:41 local = 12:41Z.

## Bottom line
Cause NOT proven. Two ranked candidates remain that read-only evidence cannot separate; one cheap experiment does (steps below, needs the human at dalek, so I am not running it). What IS established:
1. Apple documents that the firewall tracks an app by its designated requirement (DR) and that a self-signed signer is not accepted by the firewall's anchor check "unless the verifying system has been told to trust them for Application Firewall purposes". Dalek's "bridle local signing" leaf is not trusted (below). That is the documented risk to the whole signing approach.
2. The firewall's stored entry for bridle IS the stable DR (not a cdhash), so signing "worked" at the storage level, yet new processes still prompt.
3. Time/sleep/reboot are NOT the trigger. Prompts track relaunch of listeners after the binary file was replaced.

## 1. What Apple documents (citations)
- Apple TN2206 "macOS Code Signing In Depth" https://developer.apple.com/library/tvos/technotes/tn2206/_index.html . Table 1, Application Firewall: "Allow if a trusted anchor check succeeds; otherwise prompt the user." Tracking: "Initial policy decision is verified against the application's DR." Also: "Application Firewall uses code signing for both its initial and tracking policy decisions." And: "Self-signed identities and self-created CA will not be verified as being valid for this check unless the verifying system has been told to trust them for Application Firewall purposes." (Note: the page also says "A program's DR should also be satisfied by updates... This is how the macOS code signing policy engine recognizes updates.") The note is old (pre-2016 wording); I found nothing newer from Apple restating or retracting it.
- Quinn (Apple DTS), Developer Forums, Feb 2024 https://developer.apple.com/forums/thread/747298 : "The macOS firewall identifies a program via its designated requirement (DR). This is a code signing construct that allows the system to know that version N+1 of the program is the 'same code' as version N." Ad hoc/unsigned: "That doesn't create a stable DR, so the firewall thinks that each build of your code is a completely new thing." Fix given: sign with an Apple Development identity, "the approval to accept incoming connections will stick." He says nothing about self-signed certs, reboot, sleep, rename vs in-place, or when it re-prompts.
- Apple Support, firewall settings https://support.apple.com/guide/mac-help/block-connections-to-your-mac-with-a-firewall-mh34041/mac : user-level only: prompt appears for an app not yet added/allowed; "might have a trusted certificate and therefore be allowed access". No identification mechanism.
- Apple Platform Security guide, firewall https://support.apple.com/guide/security/firewall-security-in-macos-seca0e83763f/web : only the option text "Automatically allow downloaded and signed software to receive incoming connections." Nothing on identity/tracking.
- man socketfilterfw (local, `man socketfilterfw`): lists --add/--remove/--blockapp etc. Says nothing about how apps are identified or re-prompted.
- TN3127 "Inside Code Signing: Requirements" https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements : FETCH FAILED TO RETURN BODY (WebFetch returned only the title). Quinn's post points to it for DR; I could not read it. A human can open it in a browser.
- Apple forum thread 740747 https://developer.apple.com/forums/thread/740747 : no Apple staff reply; the poster's own fix was a read-only dir making `codesign -s -` fail; after ad hoc signing the firewall choice persisted. Weak evidence that ad hoc signing alone can persist when the same file is not replaced. Not Apple-authoritative.
- NOT documented by Apple anywhere I could find: rename-into-place vs in-place overwrite; inode/path/alias use; cdhash fallback behaviour for a self-signed-but-DR-bearing app; any effect of reboot or sleep; how to trust a cert "for Application Firewall purposes" (no named policy in `security add-trusted-cert -p`). Anything said on those is folklore.
- Folklore (search hits, unverified): Jamf community posts https://community.jamf.com/t5/jamf-pro/firewall-issue-on-10-9-3-and-above/td-p/35334 ("firewall refuses to sign" since 10.9.4; unsigned binaries need a manual stop/start); Micro Focus KB https://support.microfocus.com/kb/doc.php?id=7021141 (apps that self-integrity-check are not signed by the firewall, dialog every launch). Treat as anecdote.
- Apple mail list https://lists.apple.com/archives/macnetworkprog/2015/Sep/msg00035.html ("launch agent and firewall"): FETCH FAILED, HTTP 502 Bad Gateway.
- c2 wiki (rule c2-wiki-in-searches): reachable (https://c2.com/wiki/remodel/pages/MacOsx returned 200) but I guessed titles ApplicationFirewall, CodeSigning, FirewallRule: all 404. No relevant page found; c2 predates ALF/code signing, so I did not dig further.

## 2. Evidence on dalek (all read-only)
a) The firewall's stored entry (plutil -convert of /Library/Preferences/com.apple.alf.plist to a tmp xml, parsed): applications[12] = alias to file "bridle", state 0 (allow), reqdata = compiled requirement `identifier "bridle" and certificate leaf = H"37e545f62ca15230bdd4d0adec25862c80e0e6c3"`, i.e. exactly bridle's DR. There is ONE bridle entry. Contrast: entries for ad hoc/unsigned programs (caddy x2 with different hashes, python3.10, Expo, TPVirtual) store `cdhash H"..."` requirements, and caddy has two entries (one per Homebrew version): that is Quinn's "each build is a new thing", visible on disk. So bridle's signing did change what is stored; the entry is stable across builds. (applications[13] is bridle-maj3-blocktest, state 2 = blocked: a leftover from the maj3 block test, unrelated.)
b) Current binary: ~/.cargo/bin/bridle inode 194061122, mtime 12:41:25Z, Identifier=bridle, Authority=bridle local signing, `codesign --verify --strict` "valid on disk / satisfies its Designated Requirement", DR = identifier bridle and certificate leaf H"37e5...", CDHash ef3d0ee3..., TeamIdentifier not set.
c) Trust of the leaf: `security find-identity -p codesigning -v` lists it as valid (that only means usable for signing). But `security verify-cert -c <leaf> -p basic` => CSSMERR_TP_NOT_TRUSTED (`-p codeSign` printed "certificate verification successful": ambiguous, that policy check is lenient). `security dump-trust-settings -d` (admin domain): "No Trust Settings were found"; the user domain holds only 3 unrelated certs. Cert: self-signed, CA:FALSE, KU digitalSignature, EKU codeSigning, valid 2026-10-06..2036-10-03. So per TN2206 the firewall's anchor check cannot succeed for it; nobody ever "told the system to trust it for Application Firewall purposes".
d) ALF dialog answers (`log show --predicate 'subsystem == "com.apple.ALF.ApplicationFirewall"'`; "DoAnswer: filterflag 0" = Allow; log retention only reaches back to ~11-12 02:00Z so older events are unavailable): 11-12 11:14:52-53Z x3; 11-14 12:31:37Z x1; 12:43:38Z x1; 12:44:23Z x1 with filterflag 2 (a Deny; matches the blocked maj3 entry, plausibly the maj3 test); 12:51:00.2/00.98/01.67Z x3. /var/log/appfirewall.log is empty (0 bytes). The socketfilterfw log otherwise only shows SecTrustEvaluate noise and -67062 (unsigned) errors for unsigned cargo test binaries.
e) Correlation with logs (daemon.log, bridle-ui and gateway logs):
 - 11-12 03:32:05Z upgrade/restart of the launchd jobs; gateway restarted 03:32:17Z. NO dialog was answered until 11:14:52Z (3 at once, 9 s after a 'machine slept' wake line, 7h42m later). Reading: dialogs raised at 03:32Z while the human slept, answered when they woke. 3 = the 3 listening launchd jobs (gateway 7878, bridle-ui 7405, track-web 7406). Sleep did not cause them; it only delayed the answer.
 - 11-14: bridle-ui log: SIGTERM 12:41:21Z restart (before the 12:41:24Z install, so onto the OLD file), bound 100.100.189.100:7405 at 12:41:23Z, probe 12:41:26Z "reachable (a peer connected back)": NOT blocked, no dialog. The restart of an already-allowed, unreplaced file did not prompt.
 - Then 12:41:24Z install (new inode), gateway relaunched 12:41:34Z, incident "cannot connect to its own address" 12:42:34Z, an Allow at 12:43:38Z.
 - 12:50:27Z/12:50:37Z launchd restarted gateway, mails, bridle-ui, track-web onto the (unchanged since 12:41:25Z) new file. bridle-ui log 12:50:46Z: "a peer could not connect back ... reachable=false" (BLOCKED, dialog pending). Three Allows answered 12:51:00-01Z, i.e. before the gateway's 60 s self-check (12:51:27Z), which is why no second incident.
 - So within one file version: restart of the file that already had an Allow (12:41:21 ui) did not prompt, while the first launches of the NEW file (12:50:3x ui, track-web, probably gateway again) did. The 12:50 batch had no file change in the previous 9 minutes, so "a write at 12:50" is excluded; the 12:41:24 replacement is the discriminating event.
f) `launchctl print gui/501/dev.bridle.bridle-ui` and `...track-web`: `last exit reason = OS_REASON_CODESIGNING` (gateway: last exit code 0). That is the kernel killing the PREVIOUS instance for a code-signing validity failure (launchctl list shows status -9 for both; mail jobs -15 = SIGTERM). The previous instance was the 12:41:2x one that ran on the old file across the 12:41:24 replacement. Unexplained and worth a look: a rename-into-place should leave a running old inode untouched. I did not find who sent it; the kernel/AMFI messages in the unified log for 12:41-12:51 are swamped by noise (AMFI "no CMS blob" for unsigned cargo test binaries; AppleSystemPolicy "would not allow process ... ~/.cargo/bin/bridle" for every bridle exec, ~200/min, all day: that is a log-only Gatekeeper-style message for a non-notarized self-signed binary and does not discriminate).
g) Sleep/reboot: kern.boottime 11-08 08:46:25 local; the long-running daemons (pids 680, 717, 718) are from boot and were never relaunched, so they never prompt (consistent with any hypothesis). The machine sleeps dozens of times a day (daemon.log 'clock jumped'); prompts did not follow each sleep, only the upgrade restarts. I found no evidence for a post-reboot keychain/trust loss: the identity is valid and the entry still holds the DR.

## 3. Ranked candidates
A (leading, most consistent): the firewall does not trust the self-signed signer (TN2206), so for a NEW file version it cannot resolve the stored DR entry through its trust/tracking path and treats the first listen of that file as unknown, prompting once per new listening process; after the Allow, restarts of the same file are quiet (12:41 ui; 12:43 Allow then 12:50 gateway also seems quiet if the 3 dialogs were ui, track-web and one more). Test: trust the leaf and see if replacement stops prompting (experiment 2).
B: entry keyed to the file (the alias record in the entry is an old-style Alias with file name/volume/creation date; qy4y's lead (a)): rename-into-place makes a new file and the DR match is never consulted. Test: experiment 3 (in-place overwrite on a throwaway copy) or a replacement with identical contents.
C (human's timing hypothesis, "signed after launch"): REJECTED for 11-14. The log line 'installed binary signature before restart' is at 12:50:27Z; the file was signed at 12:41:25Z (before any relaunch of it), verified valid and satisfying its DR. The 12:41:21 restarts used the old file. A and B survive; C does not.
D (sleep/reboot state loss): not supported (g).
E (cdhash fallback): the stored entry is the DR, not a cdhash, so a cdhash fallback is unlikely as the stored identity.
Unresolved: which three programs the 12:51 dialogs named (the dialog text shows the program name only). Ask the human if they remember.

## 4. Proposed experiments (NONE run; each can raise a dialog or needs the human's admin password; manager-2/aide to relay)
1. Control, same file: at dalek, with the human present, `launchctl kickstart -k gui/$(id -u)/dev.bridle.gateway` twice, no binary change. Expect no dialog on the second (file already allowed). If a dialog DOES appear on a repeat launch of an unchanged allowed file, B is out and A/trust is it (entry not honored at all). Cost: gateway down ~2 s each.
2. Trust test: `security find-certificate -c "bridle local signing" -p > /tmp/bridle-leaf.pem`, then `sudo security add-trusted-cert -d -r trustRoot -p codeSign -k /Library/Keychains/System.keychain /tmp/bridle-leaf.pem` (needs the human's password; reversible with `sudo security remove-trusted-cert -d /tmp/bridle-leaf.pem`). Then a normal self-upgrade or `bridle sign binary` on a copy named bridle renamed into place, plus kickstart of the gateway. If no new dialog: A confirmed. Risk: the leaf becomes trusted for code signing system-wide on dalek; its private key sits in the login keychain, usable unattended by design, so any local process running as the user could sign code that the OS treats as trusted for those checks (Gatekeeper still blocks quarantined downloads). Low for a single-user dev Mac, but the human should decide. Only the policy name ("codeSign" vs "basic") is a guess: Apple names no ALF policy.
3. Replacement test, throwaway: copy ~/.cargo/bin/bridle to /tmp/ptu8-fw/bridle (own dir), sign it with identifier bridle and the same identity, and run it as a listener on a high port (e.g. `bridle serve` on a scratch data dir, or any tiny listener signed the same way). Allow the prompt once. Then (a) rename a re-signed copy over it and run again, (b) overwrite it in place, re-sign, run again. This separates A from B without touching the live binary. It adds a firewall entry (removable with `socketfilterfw --remove`) and needs the human at the screen.
Also read-only and free right now if wanted: ask the human what the 12:51 dialogs were titled.

## 5. PROPOSED fix (not applied; waiting on the human)
- If experiment 2 clears the prompts (A): add a one-time `just sign-setup` step to trust the leaf in the System keychain for code signing (admin password once, the same step that already asks for the login keychain password), with `bridle sign` check and a doctor warning when the leaf is untrusted. No change to install ordering. Risk as above.
- If A fails and B holds: in-place overwrite (cp over, then re-sign in place) keeps the same file but the human called it risky, rightly: a running process mapped to a file overwritten in place gets killed with OS_REASON_CODESIGNING (the f) symptom may be exactly this class), and an exec during the write sees a torn file. Only do it with the old inode released: stop the launchd jobs first, overwrite, sign, start. That is a drain-style install, a larger change.
- If neither works: br-7bs2's fallback (listen on 127.0.0.1, tailscale serve fronts the tailnet). Needs the human's go; not proposed now.
- Independent of cause: the install is currently done as soon as the build finishes, up to ~9 min before the restart (12:41:24 vs 12:50:27). That gap meant the 12:42 gateway restart (binary-changed watcher) hit a new file mid-drain, which raised the 12:42 dialog separately. Deferring signing::install to the restart moment would at least coalesce prompts into one batch. This is a separate small ticket candidate, not a fix for the cause.
Unexplained items to chase later: the OS_REASON_CODESIGNING kills in (f).

### note · agent:ptu8 · 2026-11-14T15:11:35.005Z
research done, findings posted on the ticket thread (br-ptu8). Cause NOT proven. Apple documents (TN2206, Quinn DTS thread 747298) that the firewall tracks apps by designated requirement and that a self-signed signer fails its anchor check unless trusted for the firewall; dalek's 'bridle local signing' leaf is untrusted (CSSMERR_TP_NOT_TRUSTED, no trust settings). alf.plist holds bridle's stable DR entry (not a cdhash), yet the first launches of the replaced file prompted (12:50 batch, 3 dialogs = 3 listeners) while a same-file restart did not (12:41 bridle-ui). Rejected: timing (file signed 12:41:25Z, before any relaunch), sleep/reboot. Also: launchctl shows OS_REASON_CODESIGNING as last exit reason for bridle-ui and track-web (unexplained). Needs the human at dalek: 3 proposed experiments (control kickstart, trust the leaf via sudo security add-trusted-cert, throwaway replacement test) plus a proposed fix with risks, all on the thread. Nothing changed or merged. Failed fetches: TN3127 body (title only), lists.apple.com 2015 msg00035 (HTTP 502); c2 wiki reachable, no relevant page.

### question · external:aide · 2026-11-14T15:11:56.288Z
aide, for the human (from ptu8's findings): which experiments may run at dalek?
1. Control: restart the gateway twice with no binary change (expect no dialog on the 2nd).
2. Trust test (Apple's documented cause, TN2206): trust the "bridle local signing" cert for code signing in the System keychain (sudo, the human's password; reversible), then one upgrade. Risk: any process running as the human could sign code macOS treats as trusted on dalek.
3. Throwaway replacement test: a scratch copy on a high port, renamed-over vs overwritten in place; separates "untrusted cert" from "keyed to the file" without touching the live binary.
Aide recommends 1 then 2; 3 only if 2 doesn't stop the prompts. Also: does the human remember the names in the three 7:51 AM ET dialogs?
