//! Who is told when a project moves (br-cxjs). An orchestrator is per machine and runs in that
//! machine's `<host>-home` project (br-rnhp), so the notices of `project stop`, `project start`
//! after a take-over and `project drain` go there, not only to the moved project's own
//! `orchestrator` role, which no one watches. Best effort: a notice that cannot be sent is
//! printed and never fails the move.

use std::io::Write;
use std::process::Command;

use bridle_api::discovery;
use bridle_api::machines::MachineMap;

/// Sends one message to `to` in `project`; a trait so tests need no daemon.
pub trait Sender {
    fn send(&self, project: &str, to: &str, body: &str) -> Result<(), String>;
}

/// `bridle --project <project> send <to> <body>`: the cross-project send path, so a home on
/// another daemon is queued and retried by the outbox.
pub struct RealSender;

impl Sender for RealSender {
    fn send(&self, project: &str, to: &str, body: &str) -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let mut cmd = Command::new(exe);
        cmd.args(["--project", project, "send", to, body]);
        // Run from the home project's checkout when it is on this machine, so the send reaches its
        // daemon directly; the moved project's own daemon may be stopped already.
        if let Some(d) = discovery::list_registry()
            .into_iter()
            .find(|d| d.project == project)
        {
            cmd.current_dir(&d.repo).env_remove("BRIDLE_URL");
        }
        let out = cmd.output().map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
}

/// This machine's `<host>-home`, when `[projects]` has it here (the same test `bridle up` uses).
pub fn home_project(machines: &MachineMap) -> Option<String> {
    let host = machines.machine.name.as_deref()?;
    let home = format!("{host}-home");
    machines
        .projects
        .get(&home)
        .is_some_and(|p| p.machine == host)
        .then_some(home)
}

/// Tells each of `to` in `home`. With no home, prints that and what is told instead
/// (`fallback`), so the output says the notice went elsewhere.
pub fn tell_home(
    sender: &dyn Sender,
    home: Option<&str>,
    to: &[&str],
    body: &str,
    fallback: &str,
    out: &mut dyn Write,
) {
    let Some(home) = home else {
        let _ = writeln!(out, "no <host>-home on this machine: {fallback}");
        return;
    };
    for who in to {
        match sender.send(home, who, body) {
            Ok(()) => {
                let _ = writeln!(out, "told {who} in {home}");
            }
            Err(e) => {
                let _ = writeln!(out, "(could not tell {who} in {home}: {e})");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Fake(RefCell<Vec<(String, String, String)>>, bool);

    impl Sender for Fake {
        fn send(&self, p: &str, to: &str, body: &str) -> Result<(), String> {
            self.0.borrow_mut().push((p.into(), to.into(), body.into()));
            if self.1 { Err("down".into()) } else { Ok(()) }
        }
    }

    #[test]
    fn each_recipient_is_told_once_in_the_home() {
        let f = Fake(RefCell::default(), false);
        let mut out = Vec::new();
        let to = ["external:orchestrator", "external:aide"];
        tell_home(&f, Some("vesta-home"), &to, "left", "x", &mut out);
        let seen = f.0.borrow();
        assert_eq!(seen.len(), 2);
        assert!(seen.iter().all(|s| s.0 == "vesta-home" && s.2 == "left"));
        assert_eq!(seen[1].1, "external:aide");
    }

    #[test]
    fn no_home_falls_back_and_says_so() {
        let f = Fake(RefCell::default(), false);
        let mut out = Vec::new();
        tell_home(
            &f,
            None,
            &["external:aide"],
            "b",
            "told the project only",
            &mut out,
        );
        assert!(f.0.borrow().is_empty());
        let out = String::from_utf8(out).unwrap();
        assert!(
            out.contains("no <host>-home") && out.contains("told the project only"),
            "{out}"
        );
    }

    #[test]
    fn a_failed_send_is_printed_not_returned() {
        let f = Fake(RefCell::default(), true);
        let mut out = Vec::new();
        tell_home(&f, Some("h-home"), &["external:aide"], "b", "x", &mut out);
        assert!(String::from_utf8(out).unwrap().contains("could not tell"));
    }

    #[test]
    fn home_is_found_only_when_it_is_on_this_machine() {
        let m = |t: &str| toml::from_str::<MachineMap>(t).unwrap();
        let here = "[machine]\nname = \"vesta\"\n[projects]\nvesta-home = { machine = \"vesta\", port = 1 }\n";
        assert_eq!(home_project(&m(here)).as_deref(), Some("vesta-home"));
        let away = "[machine]\nname = \"vesta\"\n[projects]\nvesta-home = { machine = \"dalek\", port = 1 }\n";
        assert_eq!(home_project(&m(away)), None);
        assert_eq!(home_project(&m("")), None);
    }
}
