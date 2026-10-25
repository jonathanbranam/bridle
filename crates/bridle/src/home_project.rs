//! The machine's home project in the CLI (br-7ytg): the status line, the doctor check and the
//! warn-only notices of the commands that launch or address the orchestrator. The resolution
//! itself is `bridle_api::machines::machine_home`.

use std::path::Path;

use crate::error::CliError;
use bridle_api::discovery::bridle_home;
use bridle_api::machines::{MachineHome, machine_home, non_home_orchestrators};

/// Refuses a launcher of the orchestrator in `project` (None: it picks its own) unless that is
/// the machine's home. `command` is what to run instead, `{home}` standing for the home project.
/// Only new launches are refused; a running orchestrator is never touched.
pub fn refuse_launcher(project: Option<&str>, command: &str) -> Result<(), CliError> {
    launcher_refusal(&bridle_home(), project, command)
}

fn launcher_refusal(home: &Path, project: Option<&str>, command: &str) -> Result<(), CliError> {
    match machine_home(home).problem(home, project, command) {
        Some(w) => Err(CliError::Usage(w)),
        None => Ok(()),
    }
}

/// The refusal a sender gets for mail to `external:orchestrator` in `project`, if any. Only a
/// local project is judged: another machine's home is that machine's to name.
pub fn send_refusal(home: &Path, project: &str, remote: bool) -> Option<String> {
    if remote {
        return None;
    }
    let mh = machine_home(home);
    let h = mh.home.as_deref()?;
    (h != project).then(|| {
        format!(
            "'{project}' is not this machine's home project; the orchestrator's mail \
             belongs on '{h}': send with `--project {h}`"
        )
    })
}

/// One line for `bridle status`: the home and where the orchestrator's waiter is.
pub fn status_line() -> String {
    let home = bridle_home();
    let mh = machine_home(&home);
    let waiter = bridle_daemon::wake::orchestrator_waiter(&home);
    describe(&mh, waiter.as_deref())
}

fn describe(mh: &MachineHome, waiter: Option<&str>) -> String {
    let home = mh.home.as_deref().unwrap_or("none (no machine name)");
    let src = if mh.override_.is_some() {
        " (override)"
    } else {
        ""
    };
    let at = waiter.map_or("no waiter".to_string(), |w| format!("waiter on {w}"));
    format!("{home}{src}; {at}")
}

/// The doctor check: home, override, non-home projects with `[orchestrator]` enabled, waiter.
pub fn doctor_check(repo: &Path) -> crate::doctor::Check {
    let home = bridle_home();
    let mh = machine_home(&home);
    let waiter = bridle_daemon::wake::orchestrator_waiter(&home);
    let mut detail = format!("home {}", describe(&mh, waiter.as_deref()));
    let strays = non_home_orchestrators(mh.home.as_deref(), Some(repo));
    let mut problems = Vec::new();
    if let Some(w) = mh.problem(&home, None, "") {
        problems.push(w);
    }
    if !strays.is_empty() {
        problems.push(format!(
            "[orchestrator] is enabled outside the home in: {}",
            strays.join(", ")
        ));
    }
    if let (Some(w), Some(h)) = (waiter.as_deref(), mh.home.as_deref())
        && w != h
    {
        problems.push(format!("the orchestrator's waiter is on '{w}', not '{h}'"));
    }
    if problems.is_empty() {
        return crate::doctor::Check::ok("orchestrator home", detail);
    }
    detail.push_str("; ");
    detail.push_str(&problems.join("; "));
    crate::doctor::Check::warn(
        "orchestrator home",
        detail,
        "move the orchestrator to the home and set `enabled = false` in the other projects' \
         [orchestrator] (such a daemon now refuses to start an orchestrator)",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mail_to_a_non_home_project_warns_with_the_home_command() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(
            dir.path().join("config.toml"),
            "[machine]\nname = \"mbp\"\n",
        )
        .expect("write");
        let w = send_refusal(dir.path(), "bridle", false).expect("warns");
        assert!(w.contains("--project mbp-home"), "{w}");
        assert_eq!(send_refusal(dir.path(), "mbp-home", false), None);
        assert_eq!(send_refusal(dir.path(), "bridle", true), None);
    }

    fn machine_home_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(
            dir.path().join("config.toml"),
            "[machine]\nname = \"mbp\"\n[machines]\nmbp = \"h\"\n[projects]\nmbp-home = { machine = \"mbp\", port = 1 }\n",
        )
        .expect("write");
        dir
    }

    #[test]
    fn a_launcher_refuses_a_non_home_project_naming_the_command() {
        let dir = machine_home_dir();
        let e = launcher_refusal(dir.path(), Some("bridle"), "bridle up --project {home}")
            .expect_err("refuses");
        let e = e.to_string();
        assert!(e.contains("bridle up --project mbp-home"), "{e}");
        assert!(launcher_refusal(dir.path(), Some("mbp-home"), "x").is_ok());
        // `bridle up` picks the home itself.
        assert!(launcher_refusal(dir.path(), None, "bridle up").is_ok());
    }

    #[test]
    fn mail_for_another_machines_orchestrator_is_not_refused() {
        let dir = machine_home_dir();
        // A project that lives on another machine routes through the outbox to that home.
        assert_eq!(send_refusal(dir.path(), "vesta-home", true), None);
        assert_eq!(send_refusal(dir.path(), "bridle", true), None);
    }

    #[test]
    fn describe_names_home_override_and_waiter() {
        let mh = MachineHome {
            machine: Some("m".into()),
            override_: Some("base".into()),
            home: Some("base".into()),
        };
        assert_eq!(
            describe(&mh, Some("bridle")),
            "base (override); waiter on bridle"
        );
    }
}
