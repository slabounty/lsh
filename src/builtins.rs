use std::collections::HashMap;
use std::env;
use std::io::Write;

// Simple enum for builtin result
#[derive(PartialEq, Debug)]
pub enum ShellAction {
    Continue,
    Exit,
}

type BuiltinFn = fn(&[&str], &mut dyn Write, &mut dyn Write) -> ShellAction;

fn builtin_cd(args: &[&str], _: &mut dyn Write, err: &mut dyn Write) -> ShellAction {
    let target = args.get(0).copied().unwrap_or("/");
    if let Err(e) = std::env::set_current_dir(target) {
        let _ = writeln!(err, "cd: {}", e);
    }
    ShellAction::Continue
}

fn builtin_pwd(_: &[&str], out: &mut dyn Write, _: &mut dyn Write) -> ShellAction {
    if let Ok(dir) = env::current_dir() {
        writeln!(out, "{}", dir.display()).unwrap();
    }
    ShellAction::Continue
}

fn builtin_echo(args: &[&str], out: &mut dyn Write, _: &mut dyn Write) -> ShellAction {
    writeln!(out, "{}", args.join(" ")).unwrap();
    ShellAction::Continue
}

fn builtin_exit(_: &[&str], _: &mut dyn Write, _: &mut dyn Write) -> ShellAction {
    ShellAction::Exit
}

pub fn builtins() -> HashMap<&'static str, BuiltinFn> {
    let mut map: HashMap<&'static str, BuiltinFn> = HashMap::new();
    map.insert("cd", builtin_cd);
    map.insert("pwd", builtin_pwd);
    map.insert("echo", builtin_echo);
    map.insert("exit", builtin_exit);
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use serial_test::serial;


    #[test]
    fn test_exit_returns_exit_action() {
        let builtins = builtins();
        let exit_fn = builtins["exit"];
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let result = exit_fn(&[], &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Exit);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_echo_writes_to_output() {
        let builtins = builtins();
        let echo_fn = builtins["echo"];
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let result = echo_fn(&["hello", "world"], &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output.trim(), "hello world");
    }

    #[test]
    #[serial]
    fn test_pwd_prints_current_directory() {
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();
        let result = builtin_pwd(&[], &mut buf, &mut err_buf);
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
        let result = builtin_cd(&[tmp_dir.to_str().unwrap()], &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);

        // pwd should now reflect the new directory
        let mut pwd_buf = Vec::new();
        let mut err_buf = Vec::new();
        builtin_pwd(&[], &mut pwd_buf, &mut err_buf);
        let output = String::from_utf8(pwd_buf).unwrap();
        assert_eq!(output.trim(), tmp_dir.display().to_string());

        // cd back to original directory
        env::set_current_dir(&orig_dir).unwrap();
        fs::remove_dir_all(&tmp_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_cd_no_args_goes_to_root() {
        let mut buf = Vec::new();
        let mut err_buf = Vec::new();

        // Save the original dir so we can restore it
        let original_dir = std::env::current_dir().unwrap();

        let result = builtin_cd(&[], &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);

        let new_dir = std::env::current_dir().unwrap();
        assert_eq!(new_dir, std::path::Path::new("/"));

        // Clean up — go back to original dir
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
        let result = builtin_cd(&["/definitely/not/a/real/path"], &mut buf, &mut err_buf);
        assert_eq!(result, ShellAction::Continue);
        let output = String::from_utf8(err_buf).unwrap();
        println!("output = {}", output);
        assert!(output.starts_with("cd: "));
    }
}
