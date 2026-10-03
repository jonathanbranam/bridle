+++
id = "br-fc05"
title = "Set up email for bridle: SES on dev.branam.us, DNS, S3, IAM (rs7p)"
kind = "chore"
state = "claimed"
created_at = "2026-09-30T03:46:36.780Z"
updated_at = "2026-10-01T01:13:06.057867Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "human",
]
+++

The seven setup steps are in docs/tickets/open/email-and-texting-for-bridle-and-track-web-rs7p.md, section 'Setup the human does'. About an hour: AWS SES identity + DKIM, DNS records in Google Cloud DNS, S3 bucket + receipt rule, one IAM key per machine, check the work domain's DMARC, send test mails. The bridge that reads the mail is built (Mail 1-3); it goes live once this setup and a [mail] section in ~/.bridle/config.toml exist. Not urgent.
Finish with `bridle task done br-fc05`.

## Thread

### note · external:advisor · 2026-09-30T03:46:36.784Z
To-do for you: Set up email for bridle: SES on dev.branam.us, DNS, S3, IAM (rs7p). Finish it with `bridle task done br-fc05`.

### note · human · 2026-10-01T01:13:06.057Z
I am not going to have time to do that before I leave - will happen some time next week. I have travel Thr-Fri and Sun-Mon.
