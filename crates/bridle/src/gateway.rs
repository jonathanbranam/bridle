//! `bridle gateway`: runs the gateway in the foreground. Its config is read here and only
//! here, so a bad `[gateway]` section can't affect any other command.
//! See docs/design/human-web-ui.md.

use anyhow::Context;
use bridle_gateway::GatewayConfig;

use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::cli::{Cli, GatewayArgs, GatewayCommand, GatewayInstallArgs};
use crate::error::CliError;
use crate::launchd::xml_escape;
use crate::render;
use crate::systemd::quote;

pub async fn run(cli: &Cli, args: &GatewayArgs) -> Result<(), CliError> {
    match &args.command {
        Some(GatewayCommand::HashPassword) => return hash_password(),
        Some(GatewayCommand::Install(a)) => return install(cli, a),
        None => {}
    }
    let home = bridle_api::discovery::bridle_home();
    let config = GatewayConfig::load(&home).map_err(anyhow::Error::from)?;
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .try_init()
        .ok();
    let listener = bridle_gateway::bind(&config)
        .await
        .with_context(|| format!("binding the gateway to {}", config.bind))?;
    // Not part of `serve`: its tests must not poll the developer's real daemons.
    let store = bridle_gateway::collect::Store::open(&bridle_gateway::collect::store_path(&home))
        .context("opening the interactions store")?;
    let _collector = bridle_gateway::collect::spawn(store.clone());
    let interactions = bridle_gateway::report::Interactions {
        store,
        config: config.interactions,
    };
    bridle_gateway::serve(listener, config.login, config.ui, interactions)
        .await
        .context("gateway")?;
    Ok(())
}

/// Reads from stdin, not an argument, so the password stays out of shell history and `ps`.
fn hash_password() -> Result<(), CliError> {
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("reading the password from stdin")?;
    let password = line.trim_end_matches(['\r', '\n']);
    if password.is_empty() {
        return Err(anyhow::anyhow!("no password on stdin").into());
    }
    let hash = bridle_gateway::auth::hash_password(password)
        .map_err(|e| anyhow::anyhow!("hashing: {e}"))?;
    println!("{hash}");
    Ok(())
}

const LABEL: &str = "dev.bridle.gateway";

struct ServiceEnv {
    exe: PathBuf,
    home: String,
    path: String,
    /// Set only when `$BRIDLE_HOME` overrides `~/.bridle`, so the service reads the same config.
    bridle_home: Option<String>,
    log: PathBuf,
}

/// The gateway is per machine, not per project, so its unit is named for no project and logs
/// under the bridle home rather than a workspace.
fn install(cli: &Cli, a: &GatewayInstallArgs) -> Result<(), CliError> {
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| anyhow!("$HOME is not set"))?);
    let bridle_home = bridle_api::discovery::bridle_home();
    let env = ServiceEnv {
        exe: std::env::current_exe().context("locating the bridle binary")?,
        home: home.to_string_lossy().into_owned(),
        path: std::env::var("PATH").unwrap_or_default(),
        bridle_home: std::env::var("BRIDLE_HOME").ok().filter(|s| !s.is_empty()),
        log: bridle_home.join("gateway.log"),
    };
    let out = match std::env::consts::OS {
        "macos" => write_unit(
            &home.join(format!("Library/LaunchAgents/{LABEL}.plist")),
            &render_plist(&env),
            a.force,
            |p| {
                (
                    format!("launchctl bootstrap gui/$(id -u) {}", p.display()),
                    format!("launchctl bootout gui/$(id -u)/{LABEL}"),
                )
            },
        )?,
        "linux" => {
            let config_home = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            write_unit(
                &config_home.join("systemd/user/bridle-gateway.service"),
                &render_systemd(&env),
                a.force,
                |_| {
                    (
                        "systemctl --user daemon-reload && systemctl --user enable --now bridle-gateway.service".into(),
                        "systemctl --user disable --now bridle-gateway.service".into(),
                    )
                },
            )?
        }
        os => {
            return Err(
                anyhow!("bridle gateway install supports macOS and Linux, not {os}").into(),
            );
        }
    };
    if cli.json {
        render::print_json(&out)?;
    } else {
        println!("{}", out["message"].as_str().unwrap_or_default());
    }
    Ok(())
}

fn write_unit(
    path: &Path,
    text: &str,
    force: bool,
    commands: impl Fn(&Path) -> (String, String),
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
    let (load, unload) = commands(path);
    let message = format!(
        "wrote {}\nnothing was started; the gateway needs a [gateway] login in ~/.bridle/config.toml first; to load it:\n  {load}\nto unload:\n  {unload}",
        path.display()
    );
    Ok(serde_json::json!({"unit": path, "load": load, "unload": unload, "message": message}))
}

fn render_plist(env: &ServiceEnv) -> String {
    let x = |s: &str| xml_escape(s);
    let exe = x(&env.exe.to_string_lossy());
    let log = x(&env.log.to_string_lossy());
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
    // KeepAlive only on a crash (non-zero exit), like the daemons': a deliberate stop stays stopped.
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>gateway</string>
  </array>
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
{bridle_home}  </dict>
</dict>
</plist>
"#,
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
        "[Unit]\nDescription=bridle gateway (the human web UI)\nAfter=network-online.target\n\n\
         [Service]\nExecStart={exe} gateway\nRestart=on-failure\nRestartSec=5\n\
         Environment={path}\nEnvironment={home}\n{bridle_home}\
         StandardOutput=append:{log}\nStandardError=append:{log}\n\n\
         [Install]\nWantedBy=default.target\n",
        exe = quote(&env.exe.to_string_lossy()),
        path = quote(&format!("PATH={}", env.path)),
        home = quote(&format!("HOME={}", env.home)),
        log = env.log.display(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(bridle_home: Option<&str>) -> ServiceEnv {
        ServiceEnv {
            exe: PathBuf::from("/opt/bin/bridle"),
            home: "/Users/h & co".into(),
            path: "/usr/bin:/bin".into(),
            bridle_home: bridle_home.map(String::from),
            log: PathBuf::from("/Users/h/.bridle/gateway.log"),
        }
    }

    #[test]
    fn plist_runs_gateway_and_restarts_on_crash() {
        let t = render_plist(&env(Some("/alt")));
        assert!(t.contains("<string>dev.bridle.gateway</string>"));
        assert!(t.contains("<string>/opt/bin/bridle</string>\n    <string>gateway</string>"));
        assert!(t.contains("<key>SuccessfulExit</key>\n    <false/>"));
        assert!(t.contains("<key>RunAtLoad</key>"));
        assert!(t.contains("h &amp; co"));
        assert!(t.contains("<key>BRIDLE_HOME</key>"));
        assert!(!render_plist(&env(None)).contains("BRIDLE_HOME"));
    }

    #[test]
    fn unit_runs_gateway_and_restarts_on_failure() {
        let t = render_systemd(&env(Some("/alt")));
        assert!(t.contains("ExecStart=/opt/bin/bridle gateway\n"));
        assert!(t.contains("Restart=on-failure"));
        assert!(t.contains("WantedBy=default.target"));
        assert!(t.contains("append:/Users/h/.bridle/gateway.log"));
        assert!(t.contains("Environment=BRIDLE_HOME=/alt"));
        assert!(!render_systemd(&env(None)).contains("BRIDLE_HOME"));
    }

    #[test]
    fn write_unit_refuses_overwrite_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("a/b.unit");
        let cmds = |_: &Path| ("load".to_string(), "unload".to_string());
        write_unit(&p, "x", false, cmds).unwrap();
        assert!(write_unit(&p, "y", false, cmds).is_err());
        write_unit(&p, "y", true, cmds).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "y");
    }
}
