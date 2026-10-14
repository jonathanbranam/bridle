# Set up mail on a machine

*Written 2026-10-14 for the human (ticket n3kj), from the vesta and remus setup. Optional: only
for a machine whose projects should take email. What the bridge does and every `[mail]` key are in
[[docs/design/mail|the mail design]]; the one-time AWS setup (bucket, SES receipt rule, domain) is
section 3 of [[email-and-texting-for-bridle-and-track-web-rs7p|rs7p]].*

> **Status (2026-10-14):** dalek runs the only bridge today. These steps were written from the
> design, the code (`crates/bridle-mail`, `preflight_remote.rs`) and dalek's setup; not yet run
> end to end on vesta or remus (ticket n3kj tracks that).

## What is per machine and what is per project

Every machine may run a bridge for each project it owns; all of them read the same S3 bucket, and
a bridge takes a project's mail only while its machine owns the project (the design, "Following
the owner").

- **Per machine, shared by its projects:** AWS credentials (`~/.aws/`), the `[mail]` section of
  `~/.bridle/config.toml`, and `~/.bridle/mail.key` (it signs the reply-to address of question
  mails).
- **Per project on the machine:** the `external:mail` token (`credentials.toml`, `[mail]` table)
  and the bridge's service (`bridle-mail-<project>` on Linux, `dev.bridle.mail.<project>` on
  macOS).

Do the machine-wide part (steps 1 to 4) once, before any bridge runs there. Step 5 is per project.

## 1. An IAM user for this machine

One IAM user per machine (the human, 2026-10-14: "I will have separate IAM for each machine"), so
one machine's key can be turned off without touching the others. Don't copy another machine's
`~/.aws`.

Once, for all machines: a managed policy, e.g. `bridle-mail-bridge`, so the users can't drift
apart. The bridge only calls S3 ListObjectsV2, GetObject and DeleteObject and SES v2 SendEmail:

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

Optional: tighten `ses:SendEmail` to the `dev.branam.us` identity ARN (plus the `dev@branam.us`
identity while SES is in the sandbox). Add `kms:Decrypt` only if the bucket uses a
customer-managed KMS key.

Per machine, in the AWS console:

1. IAM > Users > Create user `bridle-mail-<machine>`, no console access; attach the policy.
2. The user > Security credentials > Create access key > "Application running outside AWS". Copy
   the access key ID and the secret now: the secret is shown once.

## 2. AWS credentials on the machine

The bridge reads the standard AWS chain, never bridle's files. Its service sets only `HOME` and
`BRIDLE_AS`, so the key must be in the **`[default]` profile**: a named profile is found only with
`AWS_PROFILE` set, and the service doesn't set it.

Write the files by hand (the bridge doesn't need the AWS CLI). If `~/.aws/credentials` already
exists, look at it first and add only a missing `[default]`. Paste the secret in an editor rather
than on a command line, so it stays out of shell history:

```sh
mkdir -p ~/.aws && chmod 700 ~/.aws
$EDITOR ~/.aws/credentials
#   [default]
#   aws_access_key_id = AKIA...
#   aws_secret_access_key = ...
$EDITOR ~/.aws/config
#   [default]
#   region = us-east-1
chmod 600 ~/.aws/credentials ~/.aws/config
```

`region` may come from `[mail] region` instead of `~/.aws/config`.

**Check the key works.** `bridle project preflight --to <machine>` only checks that
`~/.aws/credentials` or `~/.aws/config` exists, not that the key works. Test it with the AWS CLI
(version 2; only for this check, the bridge doesn't use it).

Install it on Linux (vesta, remus) from AWS's own zip, the human's way: download into
`~/installs/`, look, then run. Not `apt install awscli`: Ubuntu dropped that package in 24.04, and
older releases have version 1. There is no `awscli2` package; `snap install aws-cli --classic` also
works, but needs snapd. Use `awscli-exe-linux-aarch64.zip` on an ARM machine.

```sh
mkdir -p ~/installs && cd ~/installs
curl -O https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip
unzip -q awscli-exe-linux-x86_64.zip       # sudo apt install unzip, if missing
less aws/README.md                          # optional: what it installs
sudo ./aws/install                          # /usr/local/aws-cli, links /usr/local/bin/aws
aws --version                               # aws-cli/2.x
```

To update later: `sudo ./aws/install --update` from a fresh download. On macOS: `brew install
awscli` (it is version 2).

Then:

```sh
aws sts get-caller-identity                     # Arn ends in user/bridle-mail-<machine>
aws s3 ls s3://branam-bridle-mail/inbound/      # a listing or nothing; no AccessDenied
```

Without the CLI, the first bridge is the test (step 5's check).

## 3. The shared `mail.key`

One key on every machine (the human, 2026-10-14: "i can agree with the same mail.key"), so a reply
to a question mailed before a project moved still verifies after it. A bridge creates its own key
on its first run, so copy dalek's **before** any bridge runs on the new machine. If the machine
already has one, compare hashes before replacing it.

```sh
# from dalek
scp -p ~/.bridle/mail.key <machine>:.bridle/mail.key
ssh <machine> 'chmod 600 ~/.bridle/mail.key; sha256sum < ~/.bridle/mail.key'
shasum -a 256 < ~/.bridle/mail.key      # same hash (hash the raw file, not od output)
```

## 4. `[mail]` in `~/.bridle/config.toml`

Copy the whole `[mail]` section from dalek's `~/.bridle/config.toml`, unchanged. Keep `bridle`
first in `projects`: only the bridge of the first name answers mail to unknown names, so several
bridges don't each reply. A new project that should take mail is added to `projects` on every
machine.

## 5. Per project: token and bridge

**A project moved here** (`bridle project start`, [[docs/context/move-a-project|move a project]]):
nothing to do. It mints the mail token, copies `[mail]` and `mail.key` from the old machine if
this one has none, and installs and starts the bridge. It never copies AWS credentials, which is
why step 2 comes first; `project preflight --to <machine>` checks they exist.

**A project already here, or created here:** after `bridle token pair` has minted its mail token
([[docs/context/add-a-machine|add a machine]], step 3):

```sh
bridle mail install --project P      # prints the load commands; run them. On Linux:
systemctl --user daemon-reload
systemctl --user enable --now bridle-mail-P
systemctl --user is-active bridle-mail-P
```

On macOS, run the `launchctl bootstrap` command it prints.

**Check:** `tail ~/.bridle/mail-P.log` shows polls with no warning (`AccessDenied` or
`InvalidClientTokenId` means step 2). Mail `P@dev.branam.us` from an allowlisted address: it
reaches P's aide (or orchestrator), and the bucket object is gone.

## Optional: emailed answers close questions as yours

The default `[messages] answer_for_human` is only `external:orchestrator`. For a mailed reply to
close a question as the human's, add `external:mail` in the project's `.bridle/config.toml`:

```toml
[messages]
answer_for_human = ["external:orchestrator", "external:mail"]
```
