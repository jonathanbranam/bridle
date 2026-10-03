+++
id = "br-d56b"
title = "Fix ~/.bridle/config.toml: the night schedule's end needs +1d"
kind = "chore"
state = "integrated"
created_at = "2026-10-03T01:32:14.161Z"
updated_at = "2026-10-03T01:45:15.150099Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

bridle doctor fails: [[budget.schedule]] name = "night" has start = "23:00", end = "08:00". Overnight ends must say so since r5s3: change it to end = "08:00+1d". Until then the night schedule is invalid (doctor fails on every project). Only you edit ~/.bridle files. Then run bridle doctor.

## Thread

### note · external:orchestrator · 2026-10-03T01:32:14.163Z
created for the human, priority normal

### note · external:orchestrator · 2026-10-03T01:32:14.166Z
To-do for you (normal priority): Fix ~/.bridle/config.toml: the night schedule's end needs +1d. Finish it with `bridle task done br-d56b`.

### note · external:advisor · 2026-10-03T01:45:15.130Z
From the human, via advisor (2026-10-03): fixed; checked: night end is 08:00+1d.

### note · external:advisor · 2026-10-03T01:45:15.150Z
done
