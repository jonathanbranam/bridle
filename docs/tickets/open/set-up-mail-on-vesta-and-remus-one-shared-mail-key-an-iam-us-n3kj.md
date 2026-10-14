---
id: br-n3kj
title: "Set up mail on vesta and remus: one shared mail.key, an IAM user per machine"
kind: chore
created: 2026-10-14
created_by: external:advisor/nuc-rename
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-r2nr, br-rs7p, br-v7ug]
---

## The ask

The human, 2026-10-14 (to advisor nuc-rename): "I want to get mail set up for all projects on vesta and all projects on remus". On the key: "i can agree with the same mail.key but I will have separate IAM for each machine". Then: "create a todo for me that includes steps to complete mail set up for vesta and remus following this suggestion. the same mail.key but unique iam users."

## Background (advisor nuc-rename, checked 2026-10-14)

- Today only dalek runs a mail bridge (`dev.bridle.mail.bridle`). dalek's `[mail] projects` already lists the four vesta projects, so their mail waits in the bucket, and after an hour each sender is told "not delivered yet".
- What is per machine, shared by all its projects: AWS credentials, the `[mail]` section in `~/.bridle/config.toml`, and `~/.bridle/mail.key` (it signs the reply-to addresses of question mails). What is per project per machine: the `external:mail` token in `credentials.toml` (`[mail]` table) and the `bridle-mail-<project>` systemd unit.
- `bridle project start` (a project move) copies `[mail]` and `mail.key` from the old machine only when the new one has none, mints the mail token and installs and starts the bridge. It never copies AWS credentials. A bridge creates its own `mail.key` on its first run, so copy dalek's key to each machine *before* any bridge runs there. With one key everywhere, replies to questions mailed before a move still verify after it.
- The bridge only calls S3 ListObjectsV2, GetObject, DeleteObject and SES v2 SendEmail (`crates/bridle-mail`). A separate IAM user per machine is fine. Whichever identity the AWS default chain finds is used.
- vesta already has `external:mail` tokens for all four projects, and linger is on. It has no `~/.aws` and no `mail.key` yet.

## Steps

### 1. IAM (AWS console or CLI), once

Make one managed policy, e.g. `bridle-mail-bridge`, and attach it to each machine's user, so the users can't drift apart:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    { "Effect": "Allow", "Action": "s3:ListBucket",
      "Resource": "arn:aws:s3:::branam-bridle-mail",
      "Condition": { "StringLike": { "s3:prefix": ["inbound/*", "inbound/"] } } },
    { "Effect": "Allow", "Action": ["s3:GetObject", "s3:DeleteObject"],
      "Resource": "arn:aws:s3:::branam-bridle-mail/inbound/*" },
    { "Effect": "Allow", "Action": "ses:SendEmail", "Resource": "*" }
  ]
}
```

(Tighten `ses:SendEmail` to the `dev.branam.us` identity ARN, plus the `dev@branam.us` identity while SES is in the sandbox, if you like. Add `kms:Decrypt` only if the bucket uses a customer-managed KMS key.)

Create users `bridle-mail-vesta` and `bridle-mail-remus` with that policy and one access key each.

### 1a. AWS credentials on a machine (vesta and remus)

The permanent version of these steps, for any machine: [[docs/context/mail-on-a-machine|set up
mail on a machine]].

Background: [[docs/context/add-a-machine|add a machine]], step 2 (`[mail]` and "AWS credentials
from the standard chain"); [[docs/design/mail|mail design]], Config; the original AWS setup (bucket,
SES, IAM) is section 3 of [[email-and-texting-for-bridle-and-track-web-rs7p|rs7p]]. There is no
longer guide yet. Why it is done this way:

- The bridge reads the standard AWS chain, never bridle's files. Its unit sets only `HOME` and
  `BRIDLE_AS`, so the key must be in the **`[default]` profile** of `~/.aws/credentials`: a named
  profile is not found unless `AWS_PROFILE` is set, and the unit doesn't set it.
- `bridle project preflight --to <machine>` only checks that `~/.aws/credentials` or
  `~/.aws/config` exists (`preflight_remote.rs`), not that the key works. Test it yourself (step 4).

1. **Create the access key** in the AWS console: IAM > Users > `bridle-mail-<machine>` > Security
   credentials > Create access key > "Application running outside AWS". Copy the access key ID and
   secret now (the secret is shown once). No console password for these users.
2. **Write the files on the machine** (by hand; the AWS CLI is not needed for the bridge):

   ```sh
   mkdir -p ~/.aws && chmod 700 ~/.aws
   cat > ~/.aws/credentials <<'EOF'
   [default]
   aws_access_key_id = AKIA...
   aws_secret_access_key = ...
   EOF
   cat > ~/.aws/config <<'EOF'
   [default]
   region = us-east-1
   EOF
   chmod 600 ~/.aws/credentials ~/.aws/config
   ```

   If `~/.aws/credentials` already exists, look at it first and add only a missing `[default]`
   (don't overwrite another key). Paste the secret in an editor rather than on a command line, so
   it stays out of shell history.
3. **Don't copy dalek's `~/.aws`**: each machine has its own user, so one can be revoked
   (IAM > the user > deactivate the key) without touching the others.
4. **Check the key works**, with the AWS CLI if installed (on Linux from AWS's zip, not apt: see the doc, step 2;
   `brew install awscli` on macOS):

   ```sh
   aws sts get-caller-identity                     # Arn ends in user/bridle-mail-<machine>
   aws s3 ls s3://branam-bridle-mail/inbound/      # lists (or nothing), no AccessDenied
   ```

   Without the CLI, the first bridge on the machine is the test: `tail ~/.bridle/mail-<p>.log`
   shows polls with no `AccessDenied` or `InvalidClientTokenId` warning.

### 2. vesta

```sh
# from dalek: the shared key, before any bridge runs on vesta
scp -p ~/.bridle/mail.key vesta:.bridle/mail.key
ssh vesta 'chmod 600 ~/.bridle/mail.key; sha256sum < ~/.bridle/mail.key'
shasum -a 256 < ~/.bridle/mail.key      # same hash as vesta's (hash the raw file: macOS and Linux od space their output differently)
```

On vesta:

1. AWS credentials with the `bridle-mail-vesta` key, by step 1a.
2. Copy the whole `[mail]` section from dalek's `~/.bridle/config.toml` into vesta's, unchanged. Keep `bridle` first in `projects`, so that only dalek's bridge answers mail to unknown names.
3. Per project:
   ```sh
   for p in meta-notes notes dotfiles-local meta-notes-ui; do
     bridle mail install --project $p
   done
   systemctl --user daemon-reload
   for p in meta-notes notes dotfiles-local meta-notes-ui; do
     systemctl --user enable --now bridle-mail-$p; systemctl --user is-active bridle-mail-$p
   done
   ```
4. Check: `tail ~/.bridle/mail-meta-notes.log` shows polls without warnings. Mail `meta-notes@dev.branam.us` from an allowlisted address: it arrives at meta-notes' aide (or orchestrator) and the bucket object is gone.

### 3. remus

dalek's `[projects]` has no remus projects yet (ticket v7ug), so do only the machine-wide part now. Each project then gets its token and bridge from `bridle project start` when it starts or moves there.

1. If `~/.bridle/mail.key` already exists on remus, compare it with dalek's (hash as above) before replacing it. Then copy dalek's key over (mode 600).
2. AWS credentials with the `bridle-mail-remus` key, by step 1a.
3. Copy the same `[mail]` section into remus's `~/.bridle/config.toml`.
4. When a project first runs on remus: `bridle project preflight --to remus` checks the AWS credentials. `project start` installs and starts its bridge. For a project created fresh there rather than moved, run `bridle mail install --project <p>` and enable the unit as on vesta, after `token pair` has minted its mail token.

### 4. Optional: emailed answers close questions as yours

The default `[messages] answer_for_human` is only `external:orchestrator`, and no project sets it, bridle included. For a mailed reply to close a question as the human's, add `external:mail` in each project's `.bridle/config.toml`:

```toml
[messages]
answer_for_human = ["external:orchestrator", "external:mail"]
```

### Done when

vesta's four `bridle-mail-*` units are active, a test mail to one vesta project arrives, remus has the key, AWS credentials and `[mail]`, and all three machines' `mail.key` hashes match. Then `bridle ticket done <id>`.
