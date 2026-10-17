//! `bridle up`: at boot, make sure this machine's orchestrator is running in a tmux pane tagged
//! `@bridle=orchestrator` (one per machine, rule ma8e). Idempotent; honours `~/.bridle/no-autostart`.
//! `--install` writes a launchd plist / systemd user unit that runs it at login/boot (never runs
//! `launchctl`/`systemctl`/`loginctl`); `--print-tmux-hooks` prints the tmux-resurrect snippet that
//! keeps the pane tag across a tmux restore. Ticket f8f9.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, anyhow};
use bridle_api::discovery;
use bridle_api::machines::MachineMap;

use crate::cli::{Cli, UpArgs};
use crate::error::CliError;
use crate::launchd::xml_escape;
use crate::render;
use crate::systemd::quote;

const TAG: &str = "orchestrator";
const SESSION: &str = "bridle";

/// The two tmux-resurrect hooks for the user's tmux.conf. `##` survives run-shell's format
/// expansion, so the pane tag is read when the hook runs, not expanded early.
const TMUX_HOOKS: &str = r###"# bridle: keep @bridle pane tags (the orchestrator's) across tmux-resurrect
set -g @resurrect-hook-post-save-all 'tmux list-panes -a -F "##{session_name}:##{window_index}.##{pane_index} ##{@bridle}" | grep -v " $" > "$HOME/.bridle/pane-tags"'
set -g @resurrect-hook-post-restore-all 'while read -r t tag; do tmux set-option -p -t "$t" @bridle "$tag"; done < "$HOME/.bridle/pane-tags"'
"###;

fn tmux(bin: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(bin)
        .args(args)
        .output()
        .map_err(|e| format!("running {bin}: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Starts the orchestrator if no pane carries the tag. Returns what it did.
fn ensure_orchestrator(
    bridle_home: &Path,
    tmux_bin: &str,
    command: &str,
) -> anyhow::Result<String> {
    if bridle_home.join("no-autostart").exists() {
        return Ok(format!(
            "{} exists: starting nothing",
            bridle_home.join("no-autostart").display()
        ));
    }
    // A failing list-panes means no tmux server is running yet.
    let panes = tmux(
        tmux_bin,
        &["list-panes", "-a", "-F", "#{pane_id} #{@bridle}"],
    );
    if let Ok(panes) = &panes
        && let Some(id) = panes.lines().find_map(|l| {
            let (id, tag) = l.split_once(' ')?;
            (tag.trim() == TAG).then_some(id)
        })
    {
        return Ok(format!("orchestrator already running in pane {id}"));
    }
    let fmt = ["-P", "-F", "#{pane_id}"];
    let mut args = if panes.is_ok() {
        vec!["new-window", "-d"]
    } else {
        vec!["new-session", "-d", "-s", SESSION]
    };
    args.extend(fmt);
    // A shell window first, then the command typed into it (rule tmux-shell-first): a window
    // running the launcher itself closes with the session.
    let pane = tmux(tmux_bin, &args).map_err(|e| anyhow!("tmux {}: {e}", args[0]))?;
    tmux(tmux_bin, &["send-keys", "-t", &pane, command, "Enter"])
        .map_err(|e| anyhow!("tmux send-keys: {e}"))?;
    tmux(tmux_bin, &["set-option", "-p", "-t", &pane, "@bridle", TAG])
        .map_err(|e| anyhow!("tmux set-option: {e}"))?;
    Ok(format!("started the orchestrator in pane {pane}"))
}

pub fn run(cli: &Cli, a: &UpArgs) -> Result<(), CliError> {
    if a.print_tmux_hooks {
        print!("{TMUX_HOOKS}");
        return Ok(());
    }
    if a.install {
        return install(cli, a.force);
    }
    let exe = std::env::current_exe().context("locating the bridle binary")?;
    let machines = MachineMap::load(&discovery::bridle_home()).map_err(|e| anyhow!("{e}"))?;
    let (command, project_note) = orchestrator_command(&exe.to_string_lossy(), &machines);
    let message = ensure_orchestrator(&discovery::bridle_home(), "tmux", &command)?;
    let message = format!("{message} ({project_note})");
    if cli.json {
        render::print_json(&serde_json::json!({ "message": message }))?;
    } else {
        println!("{message}");
    }
    Ok(())
}

/// The command the pane runs and a note on which project it picks. The boot hook has no cwd in
/// any workspace, so the orchestrator goes to this machine's `<host>-home` when `[projects]` has
/// it here (br-rnhp); otherwise `session` resolves the project as before.
fn orchestrator_command(exe: &str, machines: &MachineMap) -> (String, String) {
    let base = format!("{} session orchestrator", shell_quote(exe));
    let home = machines.machine.name.as_deref().and_then(|host| {
        let home = format!("{host}-home");
        let here = machines
            .projects
            .get(&home)
            .is_some_and(|p| p.machine == host);
        here.then_some(home)
    });
    match home {
        Some(home) => {
            let note = format!("project {home}");
            (format!("{base} --project {}", shell_quote(&home)), note)
        }
        None => (
            base,
            "no <host>-home in [projects] for this machine: project from $BRIDLE_PROJECT or the \
             working directory"
                .to_string(),
        ),
    }
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

fn unit_path(os: &str, home: &Path) -> Result<PathBuf, CliError> {
    match os {
        "macos" => Ok(home.join("Library/LaunchAgents/dev.bridle.up.plist")),
        "linux" => {
            let config_home = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            Ok(config_home.join("systemd/user/bridle-up.service"))
        }
        os => Err(anyhow!("bridle up --install supports macOS and Linux, not {os}").into()),
    }
}

fn render_plist(exe: &str, path: &str, home: &str, log: &str) -> String {
    let x = xml_escape;
    // No KeepAlive: `up` runs once at load and exits; the orchestrator lives in tmux.
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.bridle.up</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>up</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
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
  </dict>
</dict>
</plist>
"#,
        exe = x(exe),
        path = x(path),
        home = x(home),
        log = x(log),
    )
}

fn render_systemd(exe: &str, path: &str, home: &str, log: &str) -> String {
    format!(
        "[Unit]\nDescription=bridle: start this machine's orchestrator in tmux\n\n\
         [Service]\nType=oneshot\nRemainAfterExit=yes\nExecStart={exe} up\n\
         Environment={path}\nEnvironment={home}\n\
         StandardOutput=append:{log}\nStandardError=append:{log}\n\n\
         [Install]\nWantedBy=default.target\n",
        exe = quote(exe),
        path = quote(&format!("PATH={path}")),
        home = quote(&format!("HOME={home}")),
    )
}

fn install(cli: &Cli, force: bool) -> Result<(), CliError> {
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| anyhow!("$HOME is not set"))?);
    let os = std::env::consts::OS;
    let path = unit_path(os, &home)?;
    if path.exists() && !force {
        return Err(anyhow!(
            "{} already exists; pass --force to overwrite",
            path.display()
        )
        .into());
    }
    let exe = std::env::current_exe().context("locating the bridle binary")?;
    let (exe, home_s) = (exe.to_string_lossy(), home.to_string_lossy());
    let env_path = std::env::var("PATH").unwrap_or_default();
    let log = discovery::bridle_home().join("up.log");
    let log = log.to_string_lossy();
    let (text, load, unload) = if os == "macos" {
        (
            render_plist(&exe, &env_path, &home_s, &log),
            format!("launchctl bootstrap gui/$(id -u) {}", path.display()),
            "launchctl bootout gui/$(id -u)/dev.bridle.up".to_string(),
        )
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "$USER".into());
        (
            render_systemd(&exe, &env_path, &home_s, &log),
            format!(
                "systemctl --user daemon-reload && systemctl --user enable --now bridle-up.service\n  (so it runs at boot with nobody logged in:) sudo loginctl enable-linger {user}"
            ),
            "systemctl --user disable --now bridle-up.service".to_string(),
        )
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    let message = format!(
        "wrote {}\nnothing was started; to load it:\n  {load}\nto unload:\n  {unload}\nto stop it starting anything: touch ~/.bridle/no-autostart",
        path.display()
    );
    if cli.json {
        render::print_json(
            &serde_json::json!({"unit": path, "load": load, "unload": unload, "message": message}),
        )?;
    } else {
        println!("{message}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// A fake tmux over a state dir: `server` exists once started; `panes` lists "id tag" lines;
    /// every call is appended to `calls`.
    fn fake_tmux(dir: &Path) -> String {
        let script = dir.join("tmux");
        std::fs::write(
            &script,
            r#"#!/bin/sh
d=$(dirname "$0")
echo "$*" >> "$d/calls"
case "$1" in
list-panes) [ -f "$d/server" ] || { echo "no server" >&2; exit 1; }; cat "$d/panes" ;;
new-session|new-window) touch "$d/server"; echo "%9"; echo "%9 " >> "$d/panes" ;;
set-option) id="$4"; tag="$6"; sed -i.bak "s/^$id .*/$id $tag/" "$d/panes"; rm -f "$d/panes.bak" ;;
esac
"#,
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(dir.join("panes"), "").unwrap();
        script.to_string_lossy().into_owned()
    }

    fn calls(dir: &Path) -> Vec<String> {
        std::fs::read_to_string(dir.join("calls"))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    #[test]
    fn tagged_pane_is_reused_and_second_run_is_a_noop() {
        let home = tempfile::tempdir().unwrap();
        let t = tempfile::tempdir().unwrap();
        let bin = fake_tmux(t.path());
        std::fs::write(t.path().join("server"), "").unwrap();
        std::fs::write(t.path().join("panes"), "%1 \n%4 orchestrator\n").unwrap();
        let m = ensure_orchestrator(home.path(), &bin, "cmd").unwrap();
        assert!(m.contains("already running in pane %4"), "{m}");
        assert_eq!(calls(t.path()).len(), 1);
    }

    #[test]
    fn no_server_creates_a_session_and_tags_then_second_run_is_a_noop() {
        let home = tempfile::tempdir().unwrap();
        let t = tempfile::tempdir().unwrap();
        let bin = fake_tmux(t.path());
        let m = ensure_orchestrator(home.path(), &bin, "cmd").unwrap();
        assert!(m.contains("started"), "{m}");
        let c = calls(t.path());
        assert!(
            c.iter().any(|l| l.starts_with("new-session -d -s bridle")),
            "{c:?}"
        );
        assert!(
            c.iter()
                .any(|l| l == "set-option -p -t %9 @bridle orchestrator"),
            "{c:?}"
        );
        let m = ensure_orchestrator(home.path(), &bin, "cmd").unwrap();
        assert!(m.contains("already running"), "{m}");
        assert_eq!(
            calls(t.path())
                .iter()
                .filter(|l| l.starts_with("new-"))
                .count(),
            1
        );
    }

    #[test]
    fn server_without_tag_adds_a_window() {
        let home = tempfile::tempdir().unwrap();
        let t = tempfile::tempdir().unwrap();
        let bin = fake_tmux(t.path());
        std::fs::write(t.path().join("server"), "").unwrap();
        std::fs::write(t.path().join("panes"), "%1 \n").unwrap();
        ensure_orchestrator(home.path(), &bin, "cmd").unwrap();
        assert!(
            calls(t.path())
                .iter()
                .any(|l| l.starts_with("new-window -d"))
        );
        let c = calls(t.path());
        assert!(c.iter().any(|l| l == "send-keys -t %9 cmd Enter"), "{c:?}");
        assert!(c.iter().all(|l| !l.ends_with(" cmd")), "{c:?}");
    }

    #[test]
    fn kill_switch_starts_nothing() {
        let home = tempfile::tempdir().unwrap();
        std::fs::write(home.path().join("no-autostart"), "").unwrap();
        let t = tempfile::tempdir().unwrap();
        let bin = fake_tmux(t.path());
        let m = ensure_orchestrator(home.path(), &bin, "cmd").unwrap();
        assert!(m.contains("no-autostart"), "{m}");
        assert!(calls(t.path()).is_empty());
    }

    #[test]
    fn the_command_names_the_home_project_when_enrolled() {
        let enrolled = |toml: &str| toml::from_str::<MachineMap>(toml).unwrap();
        let m = enrolled(
            "[machine]\nname = \"vesta\"\n[projects]\nvesta-home = { machine = \"vesta\", port = 7500 }\n",
        );
        let (cmd, note) = orchestrator_command("/bin/bridle", &m);
        assert_eq!(
            cmd,
            "'/bin/bridle' session orchestrator --project 'vesta-home'"
        );
        assert_eq!(note, "project vesta-home");
        // Another machine's home, or none, falls back to today's command and says so.
        for toml in [
            "[machine]\nname = \"vesta\"\n[projects]\nvesta-home = { machine = \"dalek\", port = 1 }\n",
            "[machine]\nname = \"vesta\"\n",
            "",
        ] {
            let (cmd, note) = orchestrator_command("/bin/bridle", &enrolled(toml));
            assert_eq!(cmd, "'/bin/bridle' session orchestrator");
            assert!(note.contains("no <host>-home"), "{note}");
        }
    }

    #[test]
    fn units_run_up() {
        assert!(render_systemd("/b/bridle", "/bin", "/h", "/l").contains("ExecStart=/b/bridle up"));
        assert!(render_plist("/b/bridle", "/bin", "/h", "/l").contains("<string>up</string>"));
    }
}
