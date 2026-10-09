//! `bridle gateway`: runs the gateway in the foreground. Its config is read here and only
//! here, so a bad `[gateway]` section can't affect any other command.
//! See docs/design/human-web-ui.md.

use anyhow::Context;
use bridle_gateway::GatewayConfig;

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
        Some(GatewayCommand::Status) => {
            let home = bridle_api::discovery::bridle_home();
            if status(cli, &home).await? {
                return Ok(());
            }
            std::process::exit(1);
        }
        Some(GatewayCommand::Stop) => {
            return stop(cli, &bridle_api::discovery::bridle_home()).await;
        }
        Some(GatewayCommand::Restart) => return restart(cli).await,
        None => {}
    }
    let home = bridle_api::discovery::bridle_home();
    let config = GatewayConfig::load(&home).map_err(anyhow::Error::from)?;
    if !config.enabled {
        println!("the gateway is disabled ([gateway] enabled = false); not starting");
        return Ok(());
    }
    let ignored: Vec<&str> = bridle_gateway::discovery::IGNORED_ENV
        .into_iter()
        .filter(|k| std::env::var_os(k).is_some())
        .collect();
    if !ignored.is_empty() {
        eprintln!(
            "warning: ignoring {} from the starting shell; the gateway acts as the human",
            ignored.join(", ")
        );
    }
    if args.detach {
        let child_args: Vec<std::ffi::OsString> = std::env::args_os()
            .skip(1)
            .filter(|a| a != "--detach")
            .collect();
        return run_detached(cli, &home, &config, &child_args).await;
    }
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
    let exe = current_exe_path()?;
    // Written once listening; dropped (file removed) on every return below. The exec path
    // keeps the file, and the new image rewrites it.
    std::fs::create_dir_all(&home).with_context(|| format!("creating {}", home.display()))?;
    let _pid_file = bridle_gateway::PidFile::write(&home).context("writing the pid file")?;
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .context("installing the SIGTERM handler")?;
    let serving = bridle_gateway::serve(listener, config.login, config.ui, interactions);
    tokio::select! {
        r = serving => r.context("gateway")?,
        _ = sigterm.recv() => tracing::info!("SIGTERM; shutting down"),
        _ = tokio::signal::ctrl_c() => tracing::info!("SIGINT; shutting down"),
        () = bridle_gateway::binary_changed(&exe, check_interval()) => {
            tracing::info!(exe = %exe.display(), "the bridle binary changed; restarting onto it");
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(&exe)
                .args(std::env::args_os().skip(1))
                .exec();
            return Err(anyhow!("exec {} failed: {err}", exe.display()).into());
        }
    }
    Ok(())
}

/// How often the gateway looks for a replaced binary. The env var is for tests only.
fn check_interval() -> Duration {
    std::env::var("BRIDLE_GATEWAY_BINARY_CHECK_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(30))
}

/// Linux appends " (deleted)" to the path of a replaced binary; the new file is at the plain path.
fn current_exe_path() -> anyhow::Result<PathBuf> {
    let p = std::env::current_exe().context("locating the bridle binary")?;
    Ok(
        match p.to_str().and_then(|s| s.strip_suffix(" (deleted)")) {
            Some(s) => PathBuf::from(s),
            None => p,
        },
    )
}

async fn run_detached(
    cli: &Cli,
    home: &Path,
    config: &GatewayConfig,
    child_args: &[std::ffi::OsString],
) -> Result<(), CliError> {
    if config.bind.port() == 0 {
        return Err(anyhow!("--detach needs a fixed port in [gateway] bind, not 0").into());
    }
    if bridle_gateway::health_ok(config.bind).await {
        return Err(anyhow!("a gateway already answers at {}", config.bind).into());
    }
    std::fs::create_dir_all(home).with_context(|| format!("creating {}", home.display()))?;
    let log_path = home.join("gateway.log");
    let mut child = crate::serve::spawn_detached_with(&log_path, "gateway", child_args)?;
    let deadline = Instant::now() + crate::serve::DETACH_WAIT;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(anyhow!(
                "gateway exited early ({status}); tail of {}:\n{}",
                log_path.display(),
                crate::serve::tail_of_log(&log_path, 40)
            )
            .into());
        }
        if bridle_gateway::health_ok(config.bind).await {
            let url = format!("http://{}", config.bind);
            if cli.json {
                render::print_json(&serde_json::json!({"url": url, "pid": child.id()}))?;
            } else {
                println!("bridle gateway listening on {url} (pid {})", child.id());
            }
            return Ok(());
        }
        if Instant::now() >= deadline {
            eprintln!(
                "warning: the gateway (pid {}) is still starting and was left running.\nLog: {}",
                child.id(),
                log_path.display()
            );
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// The first line of `ps` for one pid: `(stat, command)`; `None` when the pid is gone or a zombie.
/// A specific pid, never a search by name (rule no-kill-by-name).
fn process_command(pid: u32) -> Option<String> {
    let out = std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "stat=,command="])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().next()?.trim();
    let (stat, command) = line.split_once(char::is_whitespace)?;
    if stat.starts_with('Z') {
        return None;
    }
    Some(command.trim().to_string())
}

fn is_gateway_command(command: &str) -> bool {
    command.contains("bridle") && command.contains("gateway")
}

/// The URL as `bridle link` and the config derive it: `public_url`, else `http://<bind>`.
fn gateway_url(home: &Path, config: &GatewayConfig) -> String {
    bridle_daemon::config::ui_base_url(home, None)
        .ok()
        .flatten()
        .unwrap_or_else(|| format!("http://{}", config.bind))
}

/// Prints the status; `true` when running. A pid file whose pid is gone is removed.
async fn status(cli: &Cli, home: &Path) -> Result<bool, CliError> {
    let config = GatewayConfig::load(home).map_err(anyhow::Error::from)?;
    let pid = bridle_gateway::read_pid(home);
    let alive = pid.filter(|p| process_command(*p).is_some_and(|c| is_gateway_command(&c)));
    let Some(pid) = alive else {
        let removed = pid.is_some() && std::fs::remove_file(bridle_gateway::pid_path(home)).is_ok();
        if cli.json {
            render::print_json(
                &serde_json::json!({"running": false, "stale_pid_file_removed": removed}),
            )?;
        } else {
            println!("not running");
            if removed {
                println!("stale pid file removed");
            }
        }
        return Ok(false);
    };
    let url = gateway_url(home, &config);
    // The build the gateway itself reports at health; the installed binary's is this one's.
    let build = bridle_gateway::health_build(config.bind).await;
    let installed = bridle_gateway::build_id(&current_exe_path()?);
    let stale = matches!((&build, &installed), (Some(b), Some(i)) if b != i);
    if cli.json {
        render::print_json(&serde_json::json!({
            "running": true, "pid": pid, "url": url, "build": build, "stale_binary": stale,
        }))?;
    } else {
        println!(
            "running pid {pid} {url} build {}{}",
            build.as_deref().unwrap_or("unknown"),
            if stale { " stale binary" } else { "" }
        );
    }
    Ok(true)
}

/// SIGTERM the recorded gateway and wait for it to go; no SIGKILL.
async fn stop(cli: &Cli, home: &Path) -> Result<(), CliError> {
    let say = |state: &str, text: &str| -> Result<(), CliError> {
        if cli.json {
            render::print_json(&serde_json::json!({"state": state}))?;
        } else {
            println!("{text}");
        }
        Ok(())
    };
    let Some(pid) = bridle_gateway::read_pid(home) else {
        return say("not_running", "not running");
    };
    let Some(command) = process_command(pid) else {
        let _ = std::fs::remove_file(bridle_gateway::pid_path(home));
        return say("not_running", "not running (stale pid file removed)");
    };
    if !is_gateway_command(&command) {
        return Err(anyhow!(
            "pid {pid} in {} is not a bridle gateway ({command}); nothing signalled, nothing removed",
            bridle_gateway::pid_path(home).display()
        )
        .into());
    }
    let status = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .context("running kill")?;
    if !status.success() {
        return Err(anyhow!("could not signal pid {pid}").into());
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while process_command(pid).is_some() {
        if Instant::now() >= deadline {
            return Err(
                anyhow!("the gateway (pid {pid}) is still running 10 s after SIGTERM").into(),
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // A clean exit removed it; a gateway that died hard leaves a stale one.
    let _ = std::fs::remove_file(bridle_gateway::pid_path(home));
    say("stopped", "stopped")
}

async fn restart(cli: &Cli) -> Result<(), CliError> {
    let home = bridle_api::discovery::bridle_home();
    stop(cli, &home).await?;
    let config = GatewayConfig::load(&home).map_err(anyhow::Error::from)?;
    if !config.enabled {
        println!("the gateway is disabled ([gateway] enabled = false); not starting");
        return Ok(());
    }
    run_detached(cli, &home, &config, &[std::ffi::OsString::from("gateway")]).await?;
    status(cli, &home).await?;
    Ok(())
}

/// Reads from stdin, not an argument, so the password stays out of shell history and `ps`.
fn hash_password() -> Result<(), CliError> {
    let password = if std::io::stdin().is_terminal() {
        eprint!("Password: ");
        let _ = std::io::Write::flush(&mut std::io::stderr());
        let pass = rpassword::read_password().context("reading password")?;
        if pass.is_empty() {
            return Err(anyhow::anyhow!("no password provided").into());
        }
        eprint!("Confirm Password: ");
        let _ = std::io::Write::flush(&mut std::io::stderr());
        let confirm = rpassword::read_password().context("reading password confirmation")?;
        if pass != confirm {
            return Err(anyhow::anyhow!("passwords do not match").into());
        }
        pass
    } else {
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .context("reading the password from stdin")?;
        let password = line.trim_end_matches(['\r', '\n']).to_string();
        if password.is_empty() {
            return Err(anyhow::anyhow!("no password on stdin").into());
        }
        password
    };
    let hash = bridle_gateway::auth::hash_password(&password)
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
