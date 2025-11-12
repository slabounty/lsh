use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ShellEnv {
    pub vars: HashMap<String, String>,
}

impl  ShellEnv {
    pub fn new() -> Self {
        Self {
            vars: std::env::vars().collect(), // start with inherited env
        }
    }

    pub fn set_var(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
    }

    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    pub fn unset_var(&mut self, key: &str) {
        self.vars.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::builtins::{builtin_set, builtin_unset};

    #[test]
    fn test_set_and_get_var() {
        let mut env = ShellEnv::new();
        let mut out = Vec::new();
        let mut err = Vec::new();

        builtin_set(&["FOO", "bar"], &mut env, &mut out, &mut err);
        assert_eq!(env.get_var("FOO"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_unset_var() {
        let mut env = ShellEnv::new();
        env.set_var("FOO", "bar");

        let mut out = Vec::new();
        let mut err = Vec::new();

        builtin_unset(&["FOO"], &mut env, &mut out, &mut err);
        assert!(env.get_var("FOO").is_none());
    }
}
