//! `bridle kill-guard`: Claude Code's PreToolUse hook for Bash that refuses
//! killing by name or pattern (rule `no-kill-by-name`, ticket 75h2). The
//! `Bash(pkill *)` / `Bash(killall *)` deny rules catch the plain forms; this
//! catches what they can't match as a prefix: `pgrep ... | xargs kill`,
//! `kill $(pgrep ...)`, and (as a backstop, since whether the deny rules split
//! compound commands is not verified here) `pkill`/`killall` anywhere in a
//! compound command. Plain `kill <pid>`, `kill $!` and `kill %1` stay allowed.
//! Pure decision over the hook input; `commands/hook.rs` does the I/O.

/// Words that run another command; the command they run is the next word.
const PREFIXES: &[&str] = &["sudo", "xargs", "command", "exec", "env", "nohup", "time"];

/// The command words of a shell line: split on separators (`; | & ( ) \` $(`
/// and newlines), skip `VAR=x` assignments and prefix words. Quoting is
/// ignored, so a quoted `a; pkill` is over-matched; that is the safe side.
fn command_words(line: &str) -> Vec<&str> {
    line.split([';', '|', '&', '(', ')', '`', '\n'])
        .filter_map(|segment| {
            segment
                .split_whitespace()
                .find(|w| !w.contains('=') && !w.starts_with('-') && !PREFIXES.contains(w))
        })
        .collect()
}

pub fn deny_reason() -> String {
    "Killing by name or pattern can hit other agents' processes (rule no-kill-by-name). \
     Stop a background task with TaskStop, a process you started with `kill $!` or \
     `kill <pid>`, and a bridle waiter by starting a new wait (once it lands: \
     `bridle agent wake --stop`)."
        .to_string()
}

/// The refusal reason when the hook input is a Bash call that kills by name.
pub fn refusal(input: &serde_json::Value) -> Option<String> {
    if input.get("tool_name")?.as_str()? != "Bash" {
        return None;
    }
    let command = input.get("tool_input")?.get("command")?.as_str()?;
    let words = command_words(command);
    let has = |name: &str| words.contains(&name);
    let by_name = has("pkill") || has("killall") || (has("pgrep") && has("kill"));
    by_name.then(deny_reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bash(command: &str) -> Option<String> {
        refusal(&serde_json::json!({"tool_name": "Bash", "tool_input": {"command": command}}))
    }

    #[test]
    fn refuses_kills_by_name() {
        for c in [
            "pkill -f 'just check'",
            "killall node",
            "pgrep -f wake | xargs kill",
            "pgrep -f wake | xargs kill -9",
            "kill $(pgrep -f wake)",
            "kill -9 `pgrep -f wake`",
            "cd x && pkill -f foo",
            "sleep 1; killall foo",
            "FOO=1 pkill foo",
            "sudo pkill foo",
            "echo hi | xargs pkill",
        ] {
            assert!(bash(c).is_some(), "should refuse: {c}");
        }
    }

    #[test]
    fn allows_kills_by_pid_and_other_commands() {
        for c in [
            "kill $!",
            "kill %1",
            "kill 1234",
            "pkill_not_a_command_but_a_path/x",
            "pgrep -f wake",
            "pgrep -f wake | wc -l",
            "grep -rn pkill docs",
            "cargo test --workspace && git status",
            "echo killall",
        ] {
            assert!(bash(c).is_none(), "should allow: {c}");
        }
    }

    #[test]
    fn ignores_other_tools_and_bad_input() {
        let edit = serde_json::json!({"tool_name": "Edit", "tool_input": {"command": "pkill x"}});
        assert!(refusal(&edit).is_none());
        assert!(refusal(&serde_json::Value::Null).is_none());
    }
}
