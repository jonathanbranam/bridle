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

### 2. vesta

```sh
# from dalek: the shared key, before any bridge runs on vesta
scp -p ~/.bridle/mail.key vesta:.bridle/mail.key
ssh vesta 'chmod 600 ~/.bridle/mail.key; sha256sum < ~/.bridle/mail.key'
shasum -a 256 < ~/.bridle/mail.key      # same hash as vesta's (hash the raw file: macOS and Linux od space their output differently)
```

On vesta:

1. `~/.aws/credentials` (mode 600) with the `bridle-mail-vesta` key under `[default]`, and `region = us-east-1` in `~/.aws/config` (or rely on `region` in `[mail]`).
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
2. `~/.aws/credentials` with the `bridle-mail-remus` key, as on vesta.
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
