//! `bridle projects preflight --to <machine>` (br-bb2n, kx5u K2): the NEW-machine half of the
//! preflight. Every item is a shell snippet run on NEW over ssh (exit 0 = ready) plus the fix to
//! run there when it is not. A few items need a value read back (the version, `[machine] name`)
//! and are compared here. The ssh runner is a trait so tests never touch a network.

use std::path::Path;
use std::process::Command;

use bridle_api::machines::MachineMap;

use crate::preflight::Outcome;
use crate::project_sync::{self, Files};
use crate::token_pair::{Fleet, Where};

/// What a snippet did on NEW: exit code (255 is ssh's own failure) and its stdout, or the last
/// line of stderr when it failed.
pub struct Ran {
    pub code: i32,
    pub text: String,
}

pub trait Remote {
    fn run(&self, host: &str, script: &str) -> Ran;
}

/// The path bridle was found at, then its version; the PATH extension is `token_pair`'s.
const VERSION_SNIPPET: &str = concat!(
    r#"PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin"; "#,
    "command -v bridle; bridle --version"
);

pub struct Ssh;

impl Remote for Ssh {
    fn run(&self, host: &str, script: &str) -> Ran {
        // BatchMode: no prompt can be answered here, so fail rather than hang.
        let mut c = Command::new("ssh");
        c.args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            host,
            script,
        ]);
        match project_sync::run_timed_raw(c, host, None, project_sync::SSH_TIMEOUT) {
            Ok((0, out, _)) => Ran {
                code: 0,
                text: String::from_utf8_lossy(&out).into_owned(),
            },
            Ok((code, _, err)) => Ran {
                code,
                text: String::from_utf8_lossy(&err)
                    .trim()
                    .lines()
                    .last()
                    .unwrap_or("ssh failed")
                    .to_string(),
            },
            Err(e) => Ran { code: 255, text: e },
        }
    }
}

/// What the checks need to know about the project and OLD.
pub struct Ctx<'a> {
    /// NEW's name (a key of `[machines]`) and its ssh host.
    pub machine: &'a str,
    pub host: &'a str,
    /// OLD's bridle version, "x.y.z".
    pub version: &'a str,
    pub origin: Option<String>,
    /// The clone's path on NEW, with `$HOME` for the home prefix (the remote shell expands it).
    pub clone: String,
    /// First word of `[commands] check`.
    pub toolchain: String,
    pub mail: bool,
    /// OLD's UTC offset, "+0000" style.
    pub offset: String,
    pub fleet: &'a Fleet,
}

fn outcome(name: &'static str, pass: bool, message: String) -> Outcome {
    Outcome {
        name,
        pass,
        message,
    }
}

fn parse_version(s: &str) -> Option<Vec<u32>> {
    let v = s.split_whitespace().last()?;
    v.split('.').map(|p| p.parse().ok()).collect()
}

/// A snippet and the fix to run on NEW when it fails.
struct Probe {
    name: &'static str,
    ok: &'static str,
    script: String,
    fix: String,
}

fn probes(c: &Ctx) -> Vec<Probe> {
    let p = |name, ok, script: &str, fix: &str| Probe {
        name,
        ok,
        script: script.to_string(),
        fix: fix.to_string(),
    };
    let mut v = vec![
        p(
            "claude login",
            "logged in",
            "claude auth status 2>/dev/null | grep -Eq '\"loggedIn\": *true'",
            "claude auth login (from a local terminal or tmux, not over ssh, on macOS)",
        ),
        p("gh auth", "logged in", "gh auth status", "gh auth login"),
        p(
            "tmux",
            "installed",
            "command -v tmux",
            "install tmux (brew install tmux / apt install tmux)",
        ),
        p(
            "toolchain",
            "found",
            &format!("command -v {}", c.toolchain),
            &format!("install `{}` (the project's [commands] check)", c.toolchain),
        ),
        p(
            "timezone",
            "same offset as this machine",
            &format!("[ \"$(date +%z)\" = \"{}\" ]", c.offset),
            "set the timezone (sudo timedatectl set-timezone <zone> / System Settings)",
        ),
        // Only WSL needs systemd and linger: elsewhere these pass.
        p(
            "wsl systemd",
            "ok or not WSL",
            "! grep -qi microsoft /proc/version 2>/dev/null || [ -d /run/systemd/system ]",
            "add `[boot]\\nsystemd=true` to /etc/wsl.conf, then `wsl --shutdown` from Windows",
        ),
        p(
            "linger",
            "ok or not Linux",
            "[ \"$(uname -s)\" != Linux ] || loginctl show-user \"$USER\" -p Linger | grep -q yes",
            "sudo loginctl enable-linger $USER",
        ),
    ];
    if c.mail {
        v.push(p(
            "aws credentials",
            "found, or mail off there",
            "grep -Eq '^[[:space:]]*mail[[:space:]]*=[[:space:]]*false' \"$HOME/.bridle/config.toml\" 2>/dev/null || [ -n \"$AWS_ACCESS_KEY_ID\" ] || [ -f \"$HOME/.aws/credentials\" ] || [ -f \"$HOME/.aws/config\" ]",
            "configure AWS credentials for the mail bridge (aws configure)",
        ));
    }
    if let Some(o) = &c.origin {
        v.push(Probe {
            name: "git push",
            ok: "origin accepts a push",
            script: format!(
                "git -C \"{}\" push --dry-run origin HEAD:refs/heads/bridle-preflight-probe",
                c.clone
            ),
            fix: format!("make git push access to {o} work (ssh key or `gh auth setup-git`)"),
        });
    }
    v
}

fn failure(c: &Ctx, detail: &str, fix: &str) -> String {
    let detail = if detail.is_empty() {
        String::new()
    } else {
        format!("{detail}; ")
    };
    format!("{detail}on {} run: {fix}", c.machine)
}

/// All NEW-side checks. `files` reads the `config.toml` files for the 2wnn comparison and for
/// NEW's `[machine] name`; `confirm` is asked before cloning the project onto NEW.
pub fn check_new(
    c: &Ctx,
    remote: &dyn Remote,
    files: &dyn Files,
    confirm: &dyn Fn(&str) -> bool,
) -> Vec<Outcome> {
    let reach = remote.run(c.host, "true");
    if reach.code != 0 {
        return vec![outcome(
            "reachable",
            false,
            format!("cannot ssh to {} ({}): {}", c.machine, c.host, reach.text),
        )];
    }
    let mut out = vec![outcome("reachable", true, format!("ssh to {}", c.host))];

    // The clone first: the push probe runs inside it.
    let clone_ok = format!("test -d \"{}/.git\"", c.clone);
    let mut have_clone = remote.run(c.host, &clone_ok).code == 0;
    if !have_clone && let Some(o) = &c.origin {
        let cmd = format!("git clone {o} \"{}\"", c.clone);
        if confirm(&format!("clone the project onto {} ({cmd})?", c.machine)) {
            have_clone = remote.run(c.host, &cmd).code == 0;
        }
        if !have_clone {
            out.push(outcome(
                "clone",
                false,
                failure(c, "project not cloned", &cmd),
            ));
        }
    }
    if have_clone {
        out.push(outcome("clone", true, format!("present at {}", c.clone)));
    }

    let ver = remote.run(c.host, VERSION_SNIPPET);
    // `command -v` prints the path first, then the version line.
    let (path, vline) = ver.text.trim().split_once('\n').unwrap_or(("", ""));
    let theirs = (ver.code == 0).then(|| parse_version(vline)).flatten();
    let ours = parse_version(c.version);
    out.push(match (theirs, ours) {
        (Some(t), Some(o)) if t >= o => outcome(
            "bridle version",
            true,
            format!(
                "{} at {path}",
                vline.trim().trim_start_matches("bridle ").trim()
            ),
        ),
        (Some(_), _) => outcome(
            "bridle version",
            false,
            failure(
                c,
                &format!("older than {}", c.version),
                "bridle upgrade (or just install)",
            ),
        ),
        _ if ver.code == 0 => outcome(
            "bridle version",
            false,
            failure(
                c,
                &format!("cannot read the version of {path}"),
                "bridle --version",
            ),
        ),
        _ => outcome(
            "bridle version",
            false,
            failure(
                c,
                &format!("bridle not found on {}", c.host),
                "install bridle (just install); looked on PATH, ~/.local/bin, ~/.cargo/bin",
            ),
        ),
    });

    for pr in probes(c) {
        if pr.name == "git push" && !have_clone {
            continue;
        }
        let r = remote.run(c.host, &pr.script);
        out.push(if r.code == 0 {
            outcome(pr.name, true, pr.ok.into())
        } else {
            outcome(pr.name, false, failure(c, "", &pr.fix))
        });
    }

    out.push(machine_name(c, files));
    out.push(tailscale(c));
    out.push(sync_compare(c, files));
    out
}

/// `[machine] name` on NEW must be NEW's own key in `[machines]`: that is the name
/// `take-over` records in `owner.toml`.
fn machine_name(c: &Ctx, files: &dyn Files) -> Outcome {
    let name = "machine name";
    let fix = format!(
        "put `[machine]\\nname = \"{0}\"` in ~/.bridle/config.toml (and {0} under [machines])",
        c.machine
    );
    let text = match files.read(&Where::Ssh(c.host.to_string())) {
        Ok(t) => t,
        Err(e) => {
            return outcome(
                name,
                false,
                failure(c, &format!("cannot read config ({e})"), &fix),
            );
        }
    };
    let have = toml::from_str::<MachineMap>(&text)
        .ok()
        .and_then(|m| m.machine.name);
    match have {
        Some(n) if n == c.machine => outcome(name, true, n),
        Some(n) => outcome(name, false, failure(c, &format!("it says '{n}'"), &fix)),
        None => outcome(name, false, failure(c, "no [machine] name", &fix)),
    }
}

/// The ssh host must resolve from here (a Tailscale name is a DNS name).
fn tailscale(c: &Ctx) -> Outcome {
    use std::net::ToSocketAddrs;
    let host = c.host.rsplit('@').next().unwrap_or(c.host);
    if (host, 22)
        .to_socket_addrs()
        .is_ok_and(|mut a| a.next().is_some())
    {
        outcome("host name", true, format!("{host} resolves"))
    } else {
        outcome(
            "host name",
            false,
            format!(
                "{host} does not resolve here; run `tailscale status` and fix its entry under [machines]"
            ),
        )
    }
}

/// The 2wnn comparison without `--apply`: NEW's `[machines]` / `[projects]` must match ours.
fn sync_compare(c: &Ctx, files: &dyn Files) -> Outcome {
    let mut buf = Vec::new();
    let ok = project_sync::sync(c.fleet, files, false, &mut buf);
    let text = String::from_utf8_lossy(&buf);
    let mine: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with(&format!("{}: ", c.machine)))
        .collect();
    let same = ok && mine.iter().all(|l| l.ends_with(": same"));
    if same && !mine.is_empty() {
        outcome("projects sync", true, "same lists".into())
    } else {
        let why = if mine.is_empty() {
            "no comparison".to_string()
        } else {
            mine.join("; ")
        };
        outcome(
            "projects sync",
            false,
            format!("{why}; run `bridle projects sync --apply` here"),
        )
    }
}

/// The clone's path on NEW as the remote shell should see it: `~` becomes `$HOME`; else the
/// given path; else OLD's path under `$HOME` kept relative to it; else (outside `$HOME`, where
/// OLD's absolute path means nothing on NEW) the documented `$HOME/work/<P>-workspace/<P>`.
fn remote_clone(abs: &Path, home: Option<&Path>, path: Option<&str>) -> String {
    if let Some(p) = path {
        return match p.strip_prefix('~') {
            Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("$HOME{rest}"),
            _ => p.to_string(),
        };
    }
    match home.and_then(|h| abs.strip_prefix(h).ok()) {
        Some(rest) => format!("$HOME/{}", rest.display()),
        None => {
            let p = abs
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into());
            format!("$HOME/work/{p}-workspace/{p}")
        }
    }
}

/// Builds the context from the repo and runs the checks. `path` names the clone on NEW.
pub fn run_to(
    repo: &Path,
    machine: &str,
    path: Option<&str>,
    map: &MachineMap,
    remote: &dyn Remote,
    files: &dyn Files,
    confirm: &dyn Fn(&str) -> bool,
) -> Result<Vec<Outcome>, String> {
    let host = map.machines.get(machine).ok_or_else(|| {
        format!("'{machine}' is not a key of [machines] in ~/.bridle/config.toml")
    })?;
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let cfg: Option<toml::Value> = std::fs::read_to_string(repo.join(".bridle/config.toml"))
        .ok()
        .and_then(|t| toml::from_str(&t).ok());
    let toolchain = cfg
        .as_ref()
        .and_then(|c| c.get("commands")?.get("check")?.as_str().map(String::from))
        .unwrap_or_else(|| "just check".into());
    let abs = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let clone = remote_clone(&abs, home.as_deref(), path);
    let offset = Command::new("date")
        .arg("+%z")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let fleet = Fleet::from_config(map);
    let ctx = Ctx {
        machine,
        host,
        version: env!("CARGO_PKG_VERSION"),
        origin: git(&["remote", "get-url", "origin"]),
        clone,
        toolchain: toolchain
            .split_whitespace()
            .next()
            .unwrap_or("just")
            .to_string(),
        mail: cfg.as_ref().is_some_and(|c| c.get("mail").is_some()),
        offset,
        fleet: &fleet,
    };
    Ok(check_new(&ctx, remote, files, confirm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Answers by substring of the script; `fail` lists snippets that exit 1.
    struct Fake {
        fail: Vec<&'static str>,
        version: &'static str,
        down: bool,
    }

    impl Remote for Fake {
        fn run(&self, _: &str, script: &str) -> Ran {
            if self.down {
                return Ran {
                    code: 255,
                    text: "Connection refused".into(),
                };
            }
            if script == VERSION_SNIPPET {
                if self.version.is_empty() {
                    return Ran {
                        code: 127,
                        text: "bridle: not found".into(),
                    };
                }
                return Ran {
                    code: 0,
                    text: format!("/home/u/.cargo/bin/bridle\n{}", self.version),
                };
            }
            let code = i32::from(self.fail.iter().any(|f| script.contains(f)));
            Ran {
                code,
                text: String::new(),
            }
        }
    }

    struct Cfg(HashMap<String, String>);
    impl Files for Cfg {
        fn read(&self, at: &Where) -> Result<String, String> {
            let Where::Ssh(h) = at else {
                return Ok(String::new());
            };
            self.0.get(h).cloned().ok_or("denied".into())
        }
        fn write(&self, _: &Where, _: &str) -> Result<(), String> {
            Ok(())
        }
    }

    const OLD: &str =
        "[machine]\nname = \"mbp\"\n[machines]\nmbp = \"localhost\"\nnuc = \"localhost\"\n";
    const NEW: &str =
        "[machine]\nname = \"nuc\"\n[machines]\nmbp = \"localhost\"\nnuc = \"localhost\"\n";

    fn fleet() -> Fleet {
        Fleet {
            local: "mbp".into(),
            hosts: [("mbp", "localhost"), ("nuc", "localhost")]
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .into(),
            placed: Default::default(),
        }
    }

    fn go(r: &Fake, new_cfg: &str, mail: bool) -> Vec<Outcome> {
        let fleet = fleet();
        let c = Ctx {
            machine: "nuc",
            host: "localhost",
            version: "0.6.3",
            origin: Some("git@x:o/p.git".into()),
            clone: "$HOME/p".into(),
            toolchain: "just".into(),
            mail,
            offset: "+0000".into(),
            fleet: &fleet,
        };
        let files = Cfg([("localhost".to_string(), new_cfg.to_string())].into());
        let _ = OLD;
        check_new(&c, r, &files, &|_| false)
    }

    fn ok() -> Fake {
        Fake {
            fail: vec![],
            version: "bridle 0.6.3",
            down: false,
        }
    }

    fn msg<'a>(o: &'a [Outcome], name: &str) -> &'a Outcome {
        o.iter()
            .find(|x| x.name == name)
            .unwrap_or_else(|| panic!("no {name}"))
    }

    #[test]
    fn all_green_except_what_is_local() {
        let o = go(&ok(), NEW, false);
        // "host name" depends on resolving localhost and "projects sync" on OLD's own file,
        // which this fake leaves empty: both are checked on their own below.
        for x in o
            .iter()
            .filter(|x| !["projects sync", "host name"].contains(&x.name))
        {
            assert!(x.pass, "{}: {}", x.name, x.message);
        }
    }

    #[test]
    fn unreachable_fails_with_the_ssh_error_and_stops() {
        let o = go(&Fake { down: true, ..ok() }, NEW, false);
        assert_eq!(o.len(), 1);
        assert!(!o[0].pass && o[0].message.contains("Connection refused"));
    }

    #[test]
    fn each_failing_probe_says_what_to_run_on_new() {
        for (snippet, name, fix) in [
            ("loggedIn", "claude login", "claude auth login"),
            ("gh auth status", "gh auth", "gh auth login"),
            ("command -v tmux", "tmux", "install tmux"),
            ("command -v just", "toolchain", "install `just`"),
            ("date +%z", "timezone", "timedatectl"),
            ("/run/systemd", "wsl systemd", "wsl.conf"),
            ("Linger", "linger", "enable-linger"),
            ("push --dry-run", "git push", "push access"),
            ("test -d", "clone", "git clone"),
        ] {
            let o = go(
                &Fake {
                    fail: vec![snippet],
                    ..ok()
                },
                NEW,
                false,
            );
            let x = msg(&o, name);
            assert!(!x.pass, "{name}");
            assert!(
                x.message.contains("on nuc run:") && x.message.contains(fix),
                "{name}: {}",
                x.message
            );
        }
    }

    #[test]
    fn aws_only_when_mail_names_the_project() {
        assert!(
            go(&ok(), NEW, false)
                .iter()
                .all(|x| x.name != "aws credentials")
        );
        let o = go(
            &Fake {
                fail: vec!["AWS_ACCESS_KEY_ID"],
                ..ok()
            },
            NEW,
            true,
        );
        assert!(!msg(&o, "aws credentials").pass);
    }

    #[test]
    fn older_or_missing_bridle_fails() {
        let o = go(
            &Fake {
                version: "bridle 0.5.9",
                ..ok()
            },
            NEW,
            false,
        );
        assert!(
            msg(&o, "bridle version")
                .message
                .contains("older than 0.6.3")
        );
        let o = go(&ok(), NEW, false);
        assert!(
            msg(&o, "bridle version")
                .message
                .contains("0.6.3 at /home/u/.cargo/bin/bridle")
        );
        let o = go(
            &Fake {
                version: "bridle 0.10.0",
                ..ok()
            },
            NEW,
            false,
        );
        assert!(msg(&o, "bridle version").pass);
        let o = go(
            &Fake {
                version: "",
                ..ok()
            },
            NEW,
            false,
        );
        assert!(!msg(&o, "bridle version").pass);
        assert!(
            msg(&o, "bridle version")
                .message
                .contains("bridle not found on")
        );
        let o = go(
            &Fake {
                version: "junk",
                ..ok()
            },
            NEW,
            false,
        );
        assert!(
            msg(&o, "bridle version")
                .message
                .contains("cannot read the version")
        );
    }

    #[test]
    fn machine_name_must_be_the_target_key() {
        let o = go(&ok(), OLD, false);
        let x = msg(&o, "machine name");
        assert!(!x.pass && x.message.contains("it says 'mbp'"));
        assert!(!msg(&go(&ok(), "", false), "machine name").pass);
    }

    #[test]
    fn differing_lists_say_to_sync() {
        let o = go(
            &ok(),
            "[machine]\nname = \"nuc\"\n[machines]\nnuc = \"localhost\"\n",
            false,
        );
        let x = msg(&o, "projects sync");
        assert!(!x.pass && x.message.contains("bridle projects sync --apply"));
    }

    #[test]
    fn remote_clone_paths() {
        let h = Path::new("/home/me");
        let rc = |abs, p| remote_clone(Path::new(abs), Some(h), p);
        assert_eq!(rc("/home/me/x/p", None), "$HOME/x/p");
        assert_eq!(rc("/Volumes/Data/work/p", None), "$HOME/work/p-workspace/p");
        assert_eq!(rc("/Volumes/Data/p", Some("~/w/p")), "$HOME/w/p");
        assert_eq!(rc("/Volumes/Data/p", Some("/srv/p")), "/srv/p");
    }

    #[test]
    fn version_parse() {
        assert_eq!(parse_version("bridle 0.6.3"), Some(vec![0, 6, 3]));
        assert_eq!(parse_version("junk"), None);
    }
}
