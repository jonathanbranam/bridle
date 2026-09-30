+++
id = "br-fc05"
title = "Set up email for bridle: SES on dev.branam.us, DNS, S3, IAM (rs7p)"
kind = "chore"
state = "claimed"
created_at = "2026-09-30T03:46:36.780Z"
updated_at = "2026-09-30T03:46:36.784055Z"
+++

The seven setup steps are in docs/questions/open/email-and-texting-for-bridle-and-track-web-rs7p.md, section 'Setup the human does'. About an hour: AWS SES identity + DKIM, DNS records in Google Cloud DNS, S3 bucket + receipt rule, one IAM key per machine, check the work domain's DMARC, send test mails. Not urgent: the bridge that reads the mail isn't built yet.

## Thread

### note · external:advisor · 2026-09-30T03:46:36.784Z
To-do for you: Set up email for bridle: SES on dev.branam.us, DNS, S3, IAM (rs7p). Finish it with `bridle task done br-fc05`.
