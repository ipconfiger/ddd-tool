use crate::commands::{CommandRegistry, DddContext, Tool};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::fs;
use std::path::Path;

pub fn run(cmd: crate::commands::SetupCmd, registry: &CommandRegistry) {
    let ctx = match DddContext::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    let ddd_binary = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            println!("Error: Failed to get current executable: {}", e);
            return;
        }
    };

    match cmd.tool {
        Tool::Claude => {
            if let Err(e) = setup_claude(&ddd_binary, &ctx.project_root, registry) {
                println!("Error: {}", e);
            }
        }
        Tool::Opencode => {
            if let Err(e) = setup_opencode(&ddd_binary, &ctx.project_root, registry) {
                println!("Error: {}", e);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptTask {
    pub name: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub prompt: String,
    pub args: Vec<Arg>,
    pub tools: Vec<String>,
    pub permissions: Vec<String>,
    #[serde(rename = "auto_confirm")]
    pub auto_confirm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Arg {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    pub required: bool,
}

fn setup_claude(ddd_binary: &Path, project_root: &Path, registry: &CommandRegistry) -> Result<()> {
    let claude_dir = project_root.join(".claude");
    let commands_dir = claude_dir.join("commands");
    fs::create_dir_all(&commands_dir)?;
    prepare_init_file(project_root, "CLAUDE")?;

    let commands = registry.all();
    for cmd in &commands {
        let name = cmd.name();
        let prompt = cmd
            .command_prompt(ddd_binary.to_string_lossy().as_ref(), cmd.name())
            .unwrap_or_default();

        let cmd_file = commands_dir.join(format!("ddd-{}.md", name));
        let content = to_string(&PromptTask {
            name: name.to_string(),
            task_type: "ai".to_string(),
            prompt: prompt,
            args: vec![],
            tools: vec!["bash".to_string(), "read".to_string(), "write".to_string()],
            permissions: vec![
                "bash".to_string(),
                "read".to_string(),
                "write".to_string(),
            ],
            auto_confirm: true,
        })?;
        fs::write(&cmd_file, content)?;
    }

    println!("Claude Code setup complete!");
    println!(
        "  Commands: .claude/commands/ddd-*.md ({} files)",
        commands.len()
    );
    println!("Restart Claude Code to use /ddd-<command> syntax");
    Ok(())
}

fn setup_opencode(
    ddd_binary: &Path,
    project_root: &Path,
    registry: &CommandRegistry,
) -> Result<()> {
    let commands_dir = project_root.join(".opencode/commands");
    let skills_dir = project_root.join(".opencode/skills");
    fs::create_dir_all(&commands_dir)?;
    fs::create_dir_all(&skills_dir)?;
    prepare_init_file(project_root, "AGENTS")?;

    let commands = registry.all();
    for cmd in &commands {
        let name = cmd.name();
        let desc = cmd.description();

        // Command file
        let cmd_file = commands_dir.join(format!("ddd-{}.md", name));
        let cmd_content = format!(
            r#"---
description: {}
agent: Sisyphus
---

!`{} {} $ARGUMENTS 2>&1`
"#,
            desc,
            ddd_binary.to_string_lossy(),
            name
        );
        fs::write(&cmd_file, cmd_content)?;

        // Skill file
        let skill_file = skills_dir.join(format!("ddd-{}.md", name));
        let skill_content = cmd
            .skill_prompt(ddd_binary.to_string_lossy().as_ref(), cmd.name())
            .unwrap_or_default();
        fs::write(&skill_file, skill_content)?;
    }

    println!("OpenCode setup complete!");
    println!(
        "  Commands: .opencode/commands/ddd-*.md ({} files)",
        commands.len()
    );
    println!(
        "  Skills: .opencode/skills/ddd-*.md ({} files)",
        commands.len()
    );
    println!("Restart OpenCode to use /ddd-<command> syntax");
    Ok(())
}

fn prepare_init_file(project_root: &Path, file_name: &str) -> Result<()> {
    let init_file = project_root.join(format!("{}.md", file_name));
    if !init_file.exists() {
        let file_txt = format!(r#"# {}.md
## **important roles**
1. **important** never edit roadmap.json in @project_docs/roadmap.json

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

"#, file_name);
        fs::write(&init_file, file_txt)?;
    }
    Ok(())
}
