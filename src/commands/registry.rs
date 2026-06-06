use std::collections::HashMap;
use super::trait_def::DddCommand;
use super::init::InitCommand;
use super::prepare::PrepareCommand;
use super::exec::ExecCommand;
use super::verify::VerifyCommand;
use super::audit::AuditCommand;
use super::final_verify::FinalVerifyCommand;
use super::confirm_phase::ConfirmCommand;
use super::archive::ArchiveCommand;
use super::report::ReportCommand;
use super::sync::SyncCommand;
use super::internal::AcceptCommand;

pub struct CommandRegistry {
    commands: HashMap<&'static str, Box<dyn DddCommand>>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            commands: HashMap::new(),
        };
        reg.register(Box::new(InitCommand));
        reg.register(Box::new(PrepareCommand));
        reg.register(Box::new(ExecCommand));
        reg.register(Box::new(VerifyCommand));
        reg.register(Box::new(AuditCommand));
        reg.register(Box::new(FinalVerifyCommand));
        reg.register(Box::new(ConfirmCommand));
        reg.register(Box::new(ArchiveCommand));
        reg.register(Box::new(ReportCommand));
        reg.register(Box::new(SyncCommand));
        reg.register(Box::new(AcceptCommand));
        reg
    }

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn DddCommand> {
        self.commands.get(name).map(|c| c.as_ref())
    }

    pub fn all(&self) -> Vec<&dyn DddCommand> {
        self.commands.values().map(|c| c.as_ref()).collect()
    }

    /// Get only CLI-visible commands (for help display and CLI dispatch)
    pub fn cli_visible_commands(&self) -> Vec<&dyn DddCommand> {
        self.commands.values()
            .filter(|c| c.is_cli_visible())
            .map(|c| c.as_ref())
            .collect()
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.commands.keys().copied().collect()
    }

    pub fn register(&mut self, cmd: Box<dyn DddCommand>) {
        let name = cmd.name();
        self.commands.insert(name, cmd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::trait_def::{CommandResult, DddCommand};
    use anyhow::Result;

    struct MockCommand {
        cmd_name: &'static str,
        cmd_desc: &'static str,
    }

    impl DddCommand for MockCommand {
        fn name(&self) -> &'static str {
            self.cmd_name
        }
        fn description(&self) -> &'static str {
            self.cmd_desc
        }
        fn command_prompt(&self, _bin: &str, _name: &str) -> Option<String> {
            Some("mock command".to_string())
        }
        fn skill_prompt(&self, _bin: &str, _name: &str) -> Option<String> {
            Some("mock skill".to_string())
        }
        fn execute(&self, _ctx: &crate::commands::DddContext, _args: &str) -> Result<CommandResult> {
            Ok(CommandResult::ok("mock"))
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = CommandRegistry::empty();
        assert!(reg.all().is_empty());
        assert!(reg.names().is_empty());
        assert!(reg.get("anything").is_none());
    }

    #[test]
    fn register_and_get_command() {
        let mut reg = CommandRegistry::empty();
        reg.register(Box::new(MockCommand {
            cmd_name: "test-cmd",
            cmd_desc: "A test command",
        }));

        let cmd = reg.get("test-cmd").expect("command should exist");
        assert_eq!(cmd.name(), "test-cmd");
        assert_eq!(cmd.description(), "A test command");
    }

    #[test]
    fn all_returns_all_registered() {
        let mut reg = CommandRegistry::empty();
        reg.register(Box::new(MockCommand {
            cmd_name: "cmd-a",
            cmd_desc: "Command A",
        }));
        reg.register(Box::new(MockCommand {
            cmd_name: "cmd-b",
            cmd_desc: "Command B",
        }));

        let all = reg.all();
        assert_eq!(all.len(), 2);
        let names: Vec<&str> = all.iter().map(|c| c.name()).collect();
        assert!(names.contains(&"cmd-a"));
        assert!(names.contains(&"cmd-b"));
    }

    #[test]
    fn names_returns_all_keys() {
        let mut reg = CommandRegistry::empty();
        reg.register(Box::new(MockCommand {
            cmd_name: "alpha",
            cmd_desc: "Alpha",
        }));
        reg.register(Box::new(MockCommand {
            cmd_name: "beta",
            cmd_desc: "Beta",
        }));

        let mut names = reg.names();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn get_unknown_returns_none() {
        let mut reg = CommandRegistry::empty();
        reg.register(Box::new(MockCommand {
            cmd_name: "known",
            cmd_desc: "Known",
        }));

        assert!(reg.get("unknown").is_none());
    }
}
