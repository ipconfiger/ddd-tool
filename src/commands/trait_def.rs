use anyhow::Result;
use crate::commands::DddContext;

/// Unified command result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub prompt: Option<String>,
}

impl CommandResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            prompt: None,
        }
    }

    pub fn ok_with_prompt(message: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            prompt: Some(prompt.into()),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            prompt: None,
        }
    }
}

/// Unified command trait — every subcommand implements this
pub trait DddCommand: Send + Sync {
    /// Command name used in CLI (e.g. "init", "prepare")
    fn name(&self) -> &'static str;

    /// Short description for help text
    fn description(&self) -> &'static str;

    /// Prompt template if this command generates one (None for operational commands)
    fn prompt_template(&self) -> Option<&'static str> {
        None
    }

    /// Whether this command should be visible in the CLI
    /// Commands returning false still generate Skills/Commands via setup,
    /// but won't appear as user-facing CLI subcommands
    fn is_cli_visible(&self) -> bool { true }

    /// Execute the command
    fn execute(&self, ctx: &DddContext, args: &str) -> Result<CommandResult>;

    /// Generate command prompt for Claude/OpenCode setup — each command defines its own
    fn command_prompt(&self, bin: &str, name: &str) -> Option<String>;

    /// Generate skill prompt for OpenCode setup — each command defines its own
    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String>;
}
