use std::process::{Command, Stdio};

use crate::builtins::{ShellAction};
use crate::environment::ShellEnv;

/// Run an external command (non-builtin)
pub fn run_external(cmd: &str, args: &[&str], env: &ShellEnv) -> ShellAction {
    match Command::new(cmd)
        .args(args)
        .env_clear()      // <-- clear inherited env first
        .envs(&env.vars)  // ← Send our environment
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(err) => {
            eprintln!("error running '{}': {}", cmd, err);
        }
    };

    ShellAction::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_external_true() {
        let env = ShellEnv::new();
        let action = run_external("true", &[], &env);

        assert_eq!(action, ShellAction::Continue);
    }

    #[test]
    fn test_run_external_missing_command() {
        let env = ShellEnv::new();
        let action = run_external("definitely_not_a_real_cmd", &[], &env);

        assert_eq!(action, ShellAction::Continue);
    }

    #[test]
    fn test_run_external_echo_to_file() {
        use tempfile::NamedTempFile;
        use std::fs;

        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();

        // Send output to file using shell redirection
        let env = ShellEnv::new();
        run_external("sh", &["-c", &format!("echo hello > {path}")], &env);

        let contents = fs::read_to_string(file).unwrap();
        assert_eq!(contents.trim(), "hello");
    }

    #[test]
    fn test_run_external_env_propagation() {
        use tempfile::NamedTempFile;
        use std::fs;

        let mut env = ShellEnv::new();
        env.vars.insert("FOO".into(), "BAR".into());

        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();

        run_external(
            "sh",
            &["-c", &format!("echo $FOO > {path}")],
            &env,
        );

        let contents = fs::read_to_string(file).unwrap();
        assert_eq!(contents.trim(), "BAR");
    }


    #[test]
    fn test_run_external_error_exit() {
        // on Unix "false" returns exit code 1
        let env = ShellEnv::new();
        let action = run_external("false", &[], &env);

        // We don't treat exit codes as fatal yet
        assert_eq!(action, ShellAction::Continue);
    }
}
