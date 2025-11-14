use std::collections::HashMap;
use std::env;
use std::io::Write;

use crate::environment::ShellEnv;

// Simple enum for builtin result
#[derive(PartialEq, Debug)]
pub enum ShellAction {
    Continue,
    Exit,
}

pub type BuiltinFn = fn(&[&str], &mut ShellEnv, &mut dyn Write, &mut dyn Write) -> ShellAction;
pub type BuiltinMap = HashMap<&'static str, BuiltinFn>;


pub fn builtin_cd(args: &[&str], env: &mut ShellEnv, _out: &mut dyn Write, err: &mut dyn Write) -> ShellAction {
    // Determine the target directory
    let target = if args.is_empty() {
        env.get_var("HOME")
            .cloned()
            .unwrap_or_else(|| "/".to_string())
    } else if args[0] == "-" {
        match env.get_var("OLDPWD") {
            Some(path) => path.clone(),
            None => {
                let _ = writeln!(err, "cd: OLDPWD not set");
                return ShellAction::Continue;
            }
        }
    } else {
        args[0].to_string()
    };

    // Save old PWD before changing
    let old_pwd = env::current_dir().unwrap();

    // Try to change directory
    if let Err(e) = env::set_current_dir(&target) {
        let _ = writeln!(err, "cd: {}", e);
        return ShellAction::Continue;
    }

    // Update environment variables
    let new_pwd = env::current_dir().unwrap();
    env.set_var("OLDPWD", &old_pwd.to_string_lossy());
    env.set_var("PWD", &new_pwd.to_string_lossy());

    ShellAction::Continue
}

fn builtin_pwd(_: &[&str], _: &mut ShellEnv, out: &mut dyn Write, _: &mut dyn Write) -> ShellAction {
    let _ = writeln!(out, "{}", std::env::current_dir().unwrap().display());
    ShellAction::Continue
}

fn builtin_echo(args: &[&str], _: &mut ShellEnv, out: &mut dyn Write, _: &mut dyn Write) -> ShellAction {
    writeln!(out, "{}", args.join(" ")).unwrap();
    ShellAction::Continue
}

fn builtin_exit(_: &[&str], _: &mut ShellEnv, _: &mut dyn Write, _: &mut dyn Write) -> ShellAction {
    ShellAction::Exit
}

pub fn builtin_set(args: &[&str], env: &mut ShellEnv, _: &mut dyn Write, err: &mut dyn Write) -> ShellAction {
    if args.len() != 2 {
        let _ = writeln!(err, "usage: set VAR VALUE");
        return ShellAction::Continue;
    }
    env.set_var(args[0], args[1]);
    ShellAction::Continue
}

pub fn builtin_unset(args: &[&str], env: &mut ShellEnv, _out: &mut dyn Write, err: &mut dyn Write) -> ShellAction {
    if args.len() != 1 {
        let _ = writeln!(err, "usage: unset VAR");
        return ShellAction::Continue;
    }
    env.unset_var(args[0]);
    ShellAction::Continue
}

pub fn builtin_env(_args: &[&str], env: &mut ShellEnv, out: &mut dyn Write, _err: &mut dyn Write) -> ShellAction {
    for (k, v) in &env.vars {
        let _ = writeln!(out, "{}={}", k, v);
    }
    ShellAction::Continue
}

pub fn builtins() -> BuiltinMap {
    let mut map: BuiltinMap = BuiltinMap::new();
    map.insert("cd", builtin_cd);
    map.insert("pwd", builtin_pwd);
    map.insert("echo", builtin_echo);
    map.insert("exit", builtin_exit);
    map.insert("set", builtin_set);
    map.insert("unset", builtin_unset);
    map.insert("env", builtin_env);
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::io::Cursor;

    use serial_test::serial;
    use tempfile::tempdir;


    #[test]
    fn test_exit_returns_exit_action() {
        let builtins = builtins();
        let exit_fn = builtins["exit"];
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut env = ShellEnv::empty();
        let result = exit_fn(&[], &mut env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Exit);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_echo_writes_to_output() {
        let builtins = builtins();
        let echo_fn = builtins["echo"];
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut env = ShellEnv::empty();
        let result = echo_fn(&["hello", "world"], &mut env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output.trim(), "hello world");
    }

    #[test]
    #[serial]
    fn test_pwd_prints_current_directory() {
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut env = ShellEnv::empty();
        let result = builtin_pwd(&[], &mut env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);

        let output = String::from_utf8(buf).unwrap();
        let cwd = env::current_dir().unwrap();
        assert_eq!(output.trim(), cwd.display().to_string());
    }

    #[test]
    #[serial]
    fn test_cd_changes_directory_and_pwd_reflects_it() {
        // Save original directory
        let orig_dir = env::current_dir().unwrap();

        // Create a temporary directory
        let tmp_dir = orig_dir.join("tmp_test_dir");
        fs::create_dir_all(&tmp_dir).unwrap();

        // cd into the new directory
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut env = ShellEnv::empty();
        let result = builtin_cd(&[tmp_dir.to_str().unwrap()], &mut env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);

        // pwd should now reflect the new directory
        let mut pwd_buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut env = ShellEnv::empty();
        builtin_pwd(&[], &mut env, &mut pwd_buf, &mut err_buf);
        let output = String::from_utf8(pwd_buf).unwrap();
        assert_eq!(output.trim(), tmp_dir.display().to_string());

        // cd back to original directory
        env::set_current_dir(&orig_dir).unwrap();
        fs::remove_dir_all(&tmp_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_cd_no_args_goes_to_home() {
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut env = ShellEnv::new();

        // Save the original dir so we can restore it
        let original_dir = std::env::current_dir().unwrap();

        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let result = builtin_cd(&[], &mut env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);

        let new_dir = std::env::current_dir().unwrap();
        assert_eq!(new_dir, std::path::Path::new(&home));

        // Clean up
        std::env::set_current_dir(&original_dir).unwrap();

        let stdout = String::from_utf8(buf).unwrap();
        let stderr = String::from_utf8(err_buf).unwrap();

        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    #[serial]
    fn test_cd_invalid_path_prints_error() {
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut env = ShellEnv::empty();
        let result = builtin_cd(&["/definitely/not/a/real/path"], &mut env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        let output = String::from_utf8(err_buf).unwrap();
        assert!(output.starts_with("cd: "));
    }


    /// Restores the current working directory when dropped.
    struct CwdGuard {
        original: PathBuf,
    }

    impl CwdGuard {
        fn new() -> Self {
            let original = env::current_dir().expect("failed to get current_dir");
            CwdGuard { original }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            // Best-effort restore; ignore errors
            let _ = env::set_current_dir(&self.original);
        }
    }

    #[test]
    #[serial]
    fn test_cd_dash_goes_to_oldpwd() {
        // Ensure we restore cwd even if the test panics
        let _guard = CwdGuard::new();

        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut shell_env = ShellEnv::empty();

        // Two temporary directories to toggle between
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();
        let path1 = dir1.path().to_path_buf();
        let path2 = dir2.path().to_path_buf();

        // Step 1: cd into dir1
        let result = builtin_cd(&[path1.to_str().unwrap()], &mut shell_env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        assert_eq!(
            std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap(),
            std::fs::canonicalize(&path1).unwrap()
        );

        // Step 2: cd into dir2
        let result = builtin_cd(&[path2.to_str().unwrap()], &mut shell_env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        assert_eq!(
            std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap(),
            std::fs::canonicalize(&path2).unwrap()
        );

        // Step 3: cd -
        let result = builtin_cd(&["-"], &mut shell_env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        assert_eq!(
            std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap(),
            std::fs::canonicalize(&path1).unwrap()
        );

        // Step 4: cd - again → back to dir2
        let result = builtin_cd(&["-"], &mut shell_env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        assert_eq!(
            std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap(),
            std::fs::canonicalize(&path2).unwrap()
        );

        // No unexpected stderr
        let stderr = String::from_utf8(err_buf).unwrap();
        assert!(stderr.is_empty());

        // _guard drops here and restores the original cwd
    }

    #[test]
    #[serial]
    fn test_cd_dash_goes_writes_to_err_old_pwd_not_set() {
        // Ensure we restore cwd even if the test panics
        let _guard = CwdGuard::new();

        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let mut shell_env = ShellEnv::empty();

        let result = builtin_cd(&["-"], &mut shell_env, &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);

        // No unexpected stderr
        let stderr = String::from_utf8(err_buf).unwrap();
        assert_eq!(stderr.trim(), "cd: OLDPWD not set");
    }

    #[test]
    fn test_builtin_env_prints_all_vars() {
        let mut env = ShellEnv::empty();
        env.set_var("USER", "testuser");
        env.set_var("HOME", "/tmp");
        env.set_var("PATH", "/usr/bin");

        let mut output = Cursor::new(Vec::new());

        let result = builtin_env(&[], &mut env, &mut output, &mut std::io::sink());
        assert!(matches!(result, ShellAction::Continue));

        let output_str = String::from_utf8(output.into_inner()).unwrap();

        // Each env var should appear as key=value followed by newline
        assert!(output_str.contains("USER=testuser"));
        assert!(output_str.contains("HOME=/tmp"));
        assert!(output_str.contains("PATH=/usr/bin"));

        // Should print one per line, so 3 lines total
        let lines: Vec<&str> = output_str.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_builtin_set_sets_the_env() {

        let mut env = ShellEnv::empty();
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();

        let result = builtin_set(&["hello", "world"], &mut env, &mut buf, &mut err_buf);
        assert!(matches!(result, ShellAction::Continue));

        assert_eq!(env.get_var("hello").unwrap(), "world");
    }

    #[test]
    fn test_builtin_set_without_args_raises_error() {

        let mut env = ShellEnv::empty();
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();

        let result = builtin_set(&[], &mut env, &mut buf, &mut err_buf);
        assert!(matches!(result, ShellAction::Continue));

        // No unexpected stderr
        let stderr = String::from_utf8(err_buf).unwrap();
        assert_eq!(stderr.trim(), "usage: set VAR VALUE");
    }

    #[test]
    fn test_builtin_unset_with_args_unsets_the_var() {

        let mut env = ShellEnv::empty();
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();

        env.set_var("hello", "world");

        let result = builtin_unset(&["hello"], &mut env, &mut buf, &mut err_buf);
        assert!(matches!(result, ShellAction::Continue));

        assert_eq!(env.get_var("hello"), None);
    }

    #[test]
    fn test_builtin_unset_without_args_raises_error() {

        let mut env = ShellEnv::empty();
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();

        let result = builtin_unset(&[], &mut env, &mut buf, &mut err_buf);
        assert!(matches!(result, ShellAction::Continue));

        // No unexpected stderr
        let stderr = String::from_utf8(err_buf).unwrap();
        assert_eq!(stderr.trim(), "usage: unset VAR");
    }
}
