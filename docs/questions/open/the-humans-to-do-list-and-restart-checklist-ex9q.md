---
id: ex9q
title: The human's to-do list, and what to do at the next restart
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [product-briefs-of-how-bridle-works-today-8awb]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> The other thing that keeps coming up is that I have things I need to do, and they are not
> being sent to me in my inbox, and they're not showing up in a task list. I don't know what we
> need. If I need tasks assigned to me, with this last version that we installed, the way that
> tokens are managed changed, and that's something I have to do. I have to go update the token
> configuration. The orchestrator, I'm sure, told me about it, but it's lost in a sea of other
> messages. A message in my inbox would be the place to start, but I can't really mark that as
> done. I can mark it as red, and that's okay. That needs to be the way this works somehow. We
> need to know for sure: "Hey human, you need to do this after the next restart." Whenever that
> restart happens, these are the things you need to do.
>
> Again, keep this solution simple, but there needs to be a little bit more help there so that
> I can remember, and so that the orchestrator always prompts me for what needs to be done at a
> restart.
>
> Something else similar is this: we just hit this thing with the SSH keys. I have to type my
> passphrase when I reboot my laptop, and that doesn't happen very often. Should we probably
> just move to Keychain to manage those, to make it easier for everyone? That needs to be a
> persistent thing that I have written down that I know I haven't done yet.

("red" is "read", from dictation.)

## What happens today (advisor, 2026-09-29, at f5cd08b)

- The human's to-dos live in prose in `docs/context/orchestrator-state.md` ("Open for the
  human: ...", "At the next restart ..."), which the human doesn't read, and in inbox
  messages, which can be marked read but not done.
- `bridle task list --claimed-by human` exists and is empty: tasks can already name the human.
- After the reboot this morning, two things the human had to do were found only when they
  broke: the token migration (t6kq; the advisor's `bridle` failed with no entry in
  `~/.bridle/credentials.toml`) and the SSH key (the agent had no identities, so `git push`
  failed).

## Shape (advisor's recommendation, KISS)

- **A human to-do is a bridle task assigned to the human**, with a one-line "when" in the
  title where it matters: `[at restart]`, `[at next reboot]` or none (whenever). The human
  marks it done with `bridle task done`. Listed with `bridle task list --claimed-by human`.
- **Whoever creates a to-do also sends one inbox message** pointing at the task, so it's seen.
  Reading the message is fine; the task is what stays open until done.
- **The orchestrator prompts at every start**: its startup step lists the human's open
  to-dos, especially the `[at restart]` ones, and tells the human first thing.
- Needs: a way to create a task assigned to the human (today claims are by the claimer).

## The human's to-dos, until the above exists

1. **[at restart] Token migration (t6kq).** Put the external tokens in
   `~/.bridle/credentials.toml` (`bridle token create advisor --project bridle`, and the same
   for the orchestrator and for each project, per
   `docs/questions/resolved/one-credentials-file-per-machine-t6kq.md`), then delete the old
   `~/.bridle-*.token` files. The orchestrator should give the exact list.
2. **[once] SSH key in the macOS Keychain**, so a reboot doesn't leave agents unable to push:
   `ssh-add --apple-use-keychain ~/.ssh/<key>` once, and in `~/.ssh/config`:
   `Host *` / `UseKeychain yes` / `AddKeysToAgent yes`. The advisor recommends it.
3. From orchestrator-state (ninth session), still open: try meta-notes mn-fbc0 and track-web's
   Space golf tasks; review the data-contracts trial (`.bridle/ADOPT-REVIEW.md`); answer rs7p.
