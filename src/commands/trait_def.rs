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

    /// Execute the command
    fn execute(&self, ctx: &DddContext, args: &str) -> Result<CommandResult>;

    /// Generate command prompt for Claude/OpenCode setup
    fn command_prompt(&self, bin: &str) -> Option<String> {
        let name = self.name();
        Some(format!(
            "使用 Bash工具 执行: {} {} $ARGUMENTS ,在命令执行完毕后，读取 stdout, 根据 stdout 制定下一步的执行任务，不要跳过或忽略任何输出信息",
            bin, name
        ))
    }

    /// Generate skill prompt for OpenCode setup
    fn skill_prompt(&self, bin: &str) -> Option<String> {
        let name = self.name();
        let desc = self.description();
        Some(format!(
            r#"---
name: "{}"
description: "{}"
---
调用 !`{} {} $ARGUMENTS 2>&1`
"#,
            name, desc, bin, name
        ))
    }
}
