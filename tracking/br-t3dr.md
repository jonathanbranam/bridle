+++
id = "br-t3dr"
title = "bridle-ui ships with each bridle release, fixed to that version, and self-upgrade moves both"
kind = "feature"
state = "pending"
created_at = "2026-10-10T17:17:50.352Z"
updated_at = "2026-10-11T21:02:10.162562Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "t3dr"
+++

docs/tickets/open/bridle-ui-ships-with-each-bridle-release-fixed-to-that-versi-t3dr.md

## Thread

### note · external:aide · 2026-10-11T18:07:00.414Z
From the human, via aide (2026-10-11 ~1:35 PM ET), on t3dr, verbatim: "A1 is good, but yes my only concern is slowing down and already slow build process; but it should be ok. it happens in CI though? that would be good." Context: the aide had listed the drawbacks of A and recommended A1 with Q2 bundled, Q3 yes with the ui.pin bump made part of cutting a release, Q4 yes. The human approved A1 (Q2). Q1 is moot: bridle-ui and bridle are both public repos (checked with gh), so no secret is needed. The aide answered the CI question: yes, the UI build runs only in release.yml on a v* tag, in parallel with the three Rust target builds; just check, worker builds and CI on main are unaffected. Q3 and Q4 were not answered explicitly: treat them as the aide recommendation, not yet confirmed by the human.

### note · external:aide · 2026-10-11T21:02:01.604Z
From the human, via aide (2026-10-11 ~4:45 PM ET), verbatim: "Q3 yes / Q4 yes". Q3: the UI reaches release machines with each bridle release, and bumping the pinned UI version is part of cutting a release. Q4: the dev machine keeps its own bridle-ui clone. Both were my recommendations and are now confirmed. A1 and Q1 were settled earlier.

### note · external:aide · 2026-10-11T21:02:10.162Z
Correction from aide: the human answered at ~5:02 PM ET, not ~4:45.
