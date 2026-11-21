//! `bridle search-guard`: Claude Code's PreToolUse hook for Bash that refuses
//! searching the whole disk or the home folder (rule `no-whole-disk-search`,
//! ticket rk4b). macOS attributes an agent's commands to the daemon that spawned
//! it, so a `find /` walks Desktop, Documents, Contacts and the rest and raises
//! privacy prompts in bridle's name. Only a root of `/`, `~`, `$HOME`, `/Users`
//! or `/Users/<name>` is refused; any narrower path (the workspace, /tmp,
//! /Volumes/Data, a worktree, `~/Library/Caches/ms-playwright`) is allowed.
//! Quoting is ignored and compound commands are split, as in `kill_guard`.
//! Pure decision over the hook input; `commands/hook.rs` does the I/O.

const PREFIXES: &[&str] = &["sudo", "xargs", "command", "exec", "env", "nohup", "time"];

/// Commands that walk a directory tree from the paths they are given.
const WALKERS: &[&str] = &["find", "fd", "du"];

fn is_wide_root(arg: &str) -> bool {
    let arg = arg.trim_matches(['"', '\'']);
    let trimmed = arg.trim_end_matches('/');
    if trimmed.is_empty() {
        return arg.starts_with('/'); // "/" or "//"
    }
    if matches!(trimmed, "~" | "$HOME" | "${HOME}" | "/Users") {
        return true;
    }
    // /Users/<name> exactly
    trimmed
        .strip_prefix("/Users/")
        .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
}

fn has_recursive_flag(args: &[&str]) -> bool {
    args.iter().any(|a| {
        *a == "--recursive"
            || (a.starts_with('-')
                && !a.starts_with("--")
                && a.chars().skip(1).any(|c| c == 'r' || c == 'R'))
    })
}

fn segment_wide(segment: &str) -> bool {
    let words: Vec<&str> = segment.split_whitespace().collect();
    let Some(i) = words
        .iter()
        .position(|w| !w.contains('=') && !PREFIXES.contains(w))
    else {
        return false;
    };
    let cmd = words[i].rsplit('/').next().unwrap_or(words[i]);
    let args = &words[i + 1..];
    let wide = || args.iter().any(|a| is_wide_root(a));
    match cmd {
        "locate" | "plocate" => true,
        "mdfind" => {
            // Spotlight searches everything unless confined with -onlyin.
            match args.iter().position(|a| *a == "-onlyin") {
                Some(p) => args.get(p + 1).is_none_or(|d| is_wide_root(d)),
                None => true,
            }
        }
        c if WALKERS.contains(&c) => wide(),
        "ls" => has_recursive_flag(args) && wide(),
        "grep" => has_recursive_flag(args) && wide(),
        "rg" => wide(),
        _ => false,
    }
}

pub fn deny_reason() -> String {
    "Searching the whole disk or home folder (find /, find ~, mdfind, locate, grep -r ~ ...) \
     makes macOS raise privacy prompts in bridle's name (rule no-whole-disk-search). Search \
     only your worktree, the workspace, /tmp or /Volumes/Data. For Playwright: use the global \
     `playwright-cli` (0.1.25, installed); its browsers are in ~/Library/Caches/ms-playwright; \
     a Node project may have its own `npx playwright`."
        .to_string()
}

/// The refusal reason when the hook input is a Bash call that searches a wide root.
pub fn refusal(input: &serde_json::Value) -> Option<String> {
    if input.get("tool_name")?.as_str()? != "Bash" {
        return None;
    }
    let command = input.get("tool_input")?.get("command")?.as_str()?;
    command
        .split([';', '|', '&', '(', ')', '`', '\n'])
        .any(segment_wide)
        .then(deny_reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bash(command: &str) -> Option<String> {
        refusal(&serde_json::json!({"tool_name": "Bash", "tool_input": {"command": command}}))
    }

    #[test]
    fn refuses_wide_searches() {
        for c in [
            "find / -name x",
            "find ~ -name x",
            "find $HOME -type d",
            "find /Users/me -name x",
            "find /Users/me/ -name x",
            "find / -name \"playwright-core\" -maxdepth 6 -type d 2>/dev/null | head -3",
            "cd /tmp && find / -name x",
            "echo hi; sudo find ~/ -name x",
            "x=$(find / -name y)",
            "mdfind playwright",
            "mdfind -onlyin ~ foo",
            "locate foo",
            "fd foo /",
            "du -sh ~",
            "ls -R /",
            "grep -rn foo ~",
            "rg foo /Users/me",
        ] {
            assert!(bash(c).is_some(), "should refuse: {c}");
        }
    }

    #[test]
    fn allows_narrow_searches() {
        for c in [
            "find . -name x",
            "find /tmp/x -name y",
            "find /Volumes/Data/work -name y",
            "find ~/Library/Caches/ms-playwright -maxdepth 1",
            "find /Users/me/work/bridle -name x",
            "mdfind -onlyin /Volumes/Data foo",
            "ls -R docs",
            "ls /",
            "ls -la ~",
            "grep -rn foo crates",
            "grep foo /",
            "rg foo",
            "cd /tmp && find . -name x",
            "echo find /",
        ] {
            assert!(bash(c).is_none(), "should allow: {c}");
        }
    }

    #[test]
    fn ignores_other_tools_and_bad_input() {
        let edit = serde_json::json!({"tool_name": "Edit", "tool_input": {"command": "find /"}});
        assert!(refusal(&edit).is_none());
        assert!(refusal(&serde_json::Value::Null).is_none());
    }
}
