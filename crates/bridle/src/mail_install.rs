//! `bridle mail install|uninstall`: a per-project LaunchAgent (macOS) or systemd user unit
//! (Linux) that runs `bridle mail run` at login/boot and restarts it on a crash. Writes and
//! removes the file only; never runs `launchctl`, `systemctl` or `loginctl`. Modelled on
//! `gateway install` (gateway.rs) and `launchd install` (launchd.rs). Ticket ezpj.

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use bridle_api::discovery;

use crate::cli::{Cli, MailInstallArgs};
use crate::error::CliError;
use crate::launchd::{project_name, xml_escape};
use crate::render;
use crate::systemd::quote;

struct ServiceEnv {
    exe: PathBuf,
    project: String,
    workdir: PathBuf,
    home: String,
    path: String,
    /// Set only when `$BRIDLE_HOME` overrides `~/.bridle`, so the service reads the same config.
    bridle_home: Option<String>,
    log: PathBuf,
}

fn label(project: &str) -> String {
    format!("dev.bridle.mail.{project}")
}

fn unit_name(project: &str) -> String {
    format!("bridle-mail-{project}.service")
}

/// The unit file for this OS, or an error naming the OS when it is neither.
fn unit_path(os: &str, home: &Path, project: &str) -> Result<PathBuf, CliError> {
    match os {
        "macos" => Ok(home.join(format!("Library/LaunchAgents/{}.plist", label(project)))),
        "linux" => {
            let config_home = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            Ok(config_home.join("systemd/user").join(unit_name(project)))
        }
        os => Err(anyhow!("bridle mail install supports macOS and Linux, not {os}").into()),
    }
}

/// Refuses, naming the fix, when `bridle mail run` could not start: no usable `[mail]` in the
/// config text, or no `mail` token for the project in the credentials file.
fn check_ready(config: &str, credentials: &Path, project: &str) -> Result<(), CliError> {
    bridle_mail::MailConfig::parse(config).map_err(|e| {
        anyhow!(
            "{e}; add a [mail] section (bucket, allow) to ~/.bridle/config.toml first: docs/design/mail.md"
        )
    })?;
    let has =
        discovery::has_credential(credentials, "mail", project).map_err(|e| anyhow!("{e}"))?;
    if !has {
        return Err(anyhow!(
            "no token for principal 'mail' on project '{project}' in {}: run `bridle token create mail --project {project}`",
            credentials.display()
        )
        .into());
    }
    Ok(())
}

pub fn install(cli: &Cli, a: &MailInstallArgs) -> Result<(), CliError> {
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| anyhow!("$HOME is not set"))?);
    let workdir = std::env::current_dir().context("current directory")?;
    let project = project_name(cli, &workdir)?;
    let os = std::env::consts::OS;
    let path = unit_path(os, &home, &project)?;
    let config_path = discovery::bridle_home().join("config.toml");
    let config = std::fs::read_to_string(&config_path).map_err(|e| {
        anyhow!(
            "reading {}: {e}; it needs a [mail] section: docs/design/mail.md",
            config_path.display()
        )
    })?;
    check_ready(&config, &discovery::credentials_path(), &project)?;
    let env = ServiceEnv {
        exe: std::env::current_exe().context("locating the bridle binary")?,
        log: discovery::bridle_home().join(format!("mail-{project}.log")),
        project,
        workdir,
        home: home.to_string_lossy().into_owned(),
        path: std::env::var("PATH").unwrap_or_default(),
        bridle_home: std::env::var("BRIDLE_HOME").ok().filter(|s| !s.is_empty()),
    };
    let text = if os == "macos" {
        render_plist(&env)
    } else {
        render_systemd(&env)
    };
    let out = write_unit(&path, &text, a.force, &commands(os, &env.project, &path))?;
    print_out(cli, &out)
}

pub fn uninstall(cli: &Cli) -> Result<(), CliError> {
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| anyhow!("$HOME is not set"))?);
    let project = project_name(cli, &std::env::current_dir().context("current directory")?)?;
    let os = std::env::consts::OS;
    let path = unit_path(os, &home, &project)?;
    if !path.exists() {
        return Err(anyhow!("{} does not exist", path.display()).into());
    }
    std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    let (_, unload) = commands(os, &project, &path);
    let message = format!(
        "removed {}\nif it is loaded, unload it (this stops the bridge):\n  {unload}",
        path.display()
    );
    print_out(
        cli,
        &serde_json::json!({"unit": path, "unload": unload, "message": message}),
    )
}

fn print_out(cli: &Cli, out: &serde_json::Value) -> Result<(), CliError> {
    if cli.json {
        render::print_json(out)?;
    } else {
        println!("{}", out["message"].as_str().unwrap_or_default());
    }
    Ok(())
}

/// (load, unload) commands to print for this OS.
fn commands(os: &str, project: &str, path: &Path) -> (String, String) {
    if os == "macos" {
        (
            format!("launchctl bootstrap gui/$(id -u) {}", path.display()),
            format!("launchctl bootout gui/$(id -u)/{}", label(project)),
        )
    } else {
        let unit = unit_name(project);
        let user = std::env::var("USER").unwrap_or_else(|_| "$USER".into());
        (
            format!(
                "systemctl --user daemon-reload && systemctl --user enable --now {unit}\n  (so it runs while you are logged out:) sudo loginctl enable-linger {user}"
            ),
            format!("systemctl --user disable --now {unit}"),
        )
    }
}

fn write_unit(
    path: &Path,
    text: &str,
    force: bool,
    (load, unload): &(String, String),
) -> Result<serde_json::Value, CliError> {
    if path.exists() && !force {
        return Err(anyhow!(
            "{} already exists; pass --force to overwrite",
            path.display()
        )
        .into());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))?;
    let message = format!(
        "wrote {}\nnothing was started; to load it:\n  {load}\nto unload:\n  {unload}",
        path.display()
    );
    Ok(serde_json::json!({"unit": path, "load": load, "unload": unload, "message": message}))
}

fn render_plist(env: &ServiceEnv) -> String {
    let x = |s: &str| xml_escape(s);
    let bridle_home = env
        .bridle_home
        .as_deref()
        .map(|h| {
            format!(
                "    <key>BRIDLE_HOME</key>\n    <string>{}</string>\n",
                x(h)
            )
        })
        .unwrap_or_default();
    let log = x(&env.log.to_string_lossy());
    // KeepAlive only on a crash (non-zero exit), like the daemons': a deliberate stop stays stopped.
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>--project</string>
    <string>{project}</string>
    <string>mail</string>
    <string>run</string>
  </array>
  <key>WorkingDirectory</key>
  <string>{wd}</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <dict>
    <key>SuccessfulExit</key>
    <false/>
  </dict>
  <key>StandardOutPath</key>
  <string>{log}</string>
  <key>StandardErrorPath</key>
  <string>{log}</string>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>{path}</string>
    <key>HOME</key>
    <string>{home}</string>
    <key>BRIDLE_AS</key>
    <string>mail</string>
{bridle_home}  </dict>
</dict>
</plist>
"#,
        label = x(&label(&env.project)),
        exe = x(&env.exe.to_string_lossy()),
        project = x(&env.project),
        wd = x(&env.workdir.to_string_lossy()),
        path = x(&env.path),
        home = x(&env.home),
    )
}

fn render_systemd(env: &ServiceEnv) -> String {
    let bridle_home = env
        .bridle_home
        .as_deref()
        .map(|h| format!("Environment={}\n", quote(&format!("BRIDLE_HOME={h}"))))
        .unwrap_or_default();
    format!(
        "[Unit]\nDescription=bridle mail bridge for {project}\nAfter=network-online.target\n\n\
         [Service]\nExecStart={exe} --project {pq} mail run\nWorkingDirectory={wd}\n\
         Restart=on-failure\nRestartSec=5\n\
         Environment={path}\nEnvironment={home}\nEnvironment=BRIDLE_AS=mail\n{bridle_home}\
         StandardOutput=append:{log}\nStandardError=append:{log}\n\n\
         [Install]\nWantedBy=default.target\n",
        project = env.project,
        pq = quote(&env.project),
        exe = quote(&env.exe.to_string_lossy()),
        wd = quote(&env.workdir.to_string_lossy()),
        path = quote(&format!("PATH={}", env.path)),
        home = quote(&format!("HOME={}", env.home)),
        log = env.log.display(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> ServiceEnv {
        ServiceEnv {
            exe: PathBuf::from("/opt/bin/bridle"),
            project: "acme".into(),
            workdir: PathBuf::from("/work/acme"),
            home: "/Users/h & co".into(),
            path: "/usr/bin:/bin".into(),
            bridle_home: None,
            log: PathBuf::from("/Users/h/.bridle/mail-acme.log"),
        }
    }

    #[test]
    fn plist_runs_mail_as_the_mail_principal_and_restarts_on_crash() {
        let t = render_plist(&env());
        assert!(t.contains("<string>dev.bridle.mail.acme</string>"));
        assert!(t.contains(
            "<string>--project</string>\n    <string>acme</string>\n    <string>mail</string>\n    <string>run</string>"
        ));
        assert!(t.contains("<key>BRIDLE_AS</key>\n    <string>mail</string>"));
        assert!(t.contains("<key>HOME</key>\n    <string>/Users/h &amp; co</string>"));
        assert!(t.contains("<key>StandardOutPath</key>\n  <string>/Users/h/.bridle/mail-acme.log"));
        assert!(t.contains("<key>RunAtLoad</key>"));
        assert!(t.contains("<key>SuccessfulExit</key>\n    <false/>"));
    }

    #[test]
    fn unit_runs_mail_as_the_mail_principal_and_restarts_on_failure() {
        let t = render_systemd(&env());
        assert!(t.contains("ExecStart=/opt/bin/bridle --project acme mail run\n"));
        assert!(t.contains("Environment=BRIDLE_AS=mail\n"));
        assert!(t.contains("Environment=\"HOME=/Users/h & co\"\n"));
        assert!(t.contains("append:/Users/h/.bridle/mail-acme.log"));
        assert!(t.contains("Restart=on-failure"));
        assert!(t.contains("WantedBy=default.target"));
    }

    #[test]
    fn linux_commands_name_the_unit_and_linger() {
        let (load, unload) = commands("linux", "acme", Path::new("/x"));
        assert!(load.contains("enable --now bridle-mail-acme.service"));
        assert!(load.contains("loginctl enable-linger"));
        assert!(unload.contains("disable --now bridle-mail-acme.service"));
    }

    #[test]
    fn other_os_is_refused() {
        assert!(unit_path("windows", Path::new("/h"), "acme").is_err());
    }

    #[test]
    fn write_unit_refuses_overwrite_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("a/b.unit");
        let cmds = ("load".to_string(), "unload".to_string());
        write_unit(&p, "x", false, &cmds).unwrap();
        assert!(write_unit(&p, "y", false, &cmds).is_err());
        write_unit(&p, "y", true, &cmds).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "y");
    }

    #[test]
    fn missing_mail_section_and_missing_token_are_refused_with_the_fix() {
        let tmp = tempfile::tempdir().unwrap();
        let creds = tmp.path().join("credentials.toml");
        let ok = "[mail]\nbucket = \"b\"\nallow = [\"me@example.com\"]\n";

        let e = check_ready("[gateway]\n", &creds, "acme")
            .unwrap_err()
            .to_string();
        assert!(e.contains("[mail]") && e.contains("config.toml"), "{e}");

        let e = check_ready(ok, &creds, "acme").unwrap_err().to_string();
        assert!(e.contains("bridle token create mail --project acme"), "{e}");

        discovery::store_credential(&creds, "mail", "acme", "tok").unwrap();
        check_ready(ok, &creds, "acme").unwrap();
        // The token is per project.
        assert!(check_ready(ok, &creds, "other").is_err());
    }
}
