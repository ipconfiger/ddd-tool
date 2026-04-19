use crate::commands::{DddContext, Tool};
use anyhow::Result;
use std::fs;
use std::path::Path;

const PUBLIC_COMMANDS: &[(&str, &str)] = &[
    ("init", "Initialize project with context"),
    ("prepare", "Prepare phrases from specs"),
    ("exec", "Execute development phase"),
    ("verify", "Verify phase成果"),
    ("fix-plan", "Generate fix plan"),
    ("fix-exec", "Execute fix plan"),
    ("archive", "Archive completed project"),
    ("report", "Generate project report"),
    ("sync", "Sync code to docs"),
    ("resume", "Resume interrupted workflow"),
];

pub fn run(cmd: crate::commands::SetupCmd) {
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
            if let Err(e) = setup_claude(&ddd_binary, &ctx.project_root) {
                println!("Error: {}", e);
            }
        }
        Tool::Opencode => {
            if let Err(e) = setup_opencode(&ddd_binary, &ctx.project_root) {
                println!("Error: {}", e);
            }
        }
    }
}

fn setup_claude(_ddd_binary: &Path, project_root: &Path) -> Result<()> {
    let claude_dir = project_root.join(".claude");
    let commands_dir = claude_dir.join("commands");
    let skills_dir = claude_dir.join("skills").join("ddd");

    // Create directories
    fs::create_dir_all(&commands_dir)?;
    fs::create_dir_all(&skills_dir)?;

    // Backup existing files
    backup_dir(&commands_dir, "ddd-", ".md")?;
    backup_file(skills_dir.join("SKILL.md").as_path())?;

    // Generate command files (10 files)
    for (name, desc) in PUBLIC_COMMANDS {
        let cmd_file = commands_dir.join(format!("ddd-{}.md", name));
        let content = format!(
            r#"---
description: "DocDriven CLI - {}"
---

Invoke the ddd skill with "{} $ARGUMENTS"
"#,
            desc, name
        );
        fs::write(&cmd_file, content)?;
    }

    // Generate main skill file
    let skill_content = format!(
        r#"---
name: ddd
description: DocDriven CLI - 文档驱动开发框架
---

# DocDriven CLI

## Available Commands

{}

## Usage

Use `/ddd-<command>` to invoke any command.

"#,
        PUBLIC_COMMANDS
            .iter()
            .map(|(name, desc)| format!("- `/ddd-{}`: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n")
    );

    fs::write(skills_dir.join("SKILL.md"), skill_content)?;

    println!("Claude Code setup complete!");
    println!("  Commands: .claude/commands/ddd-*.md ({} files)", PUBLIC_COMMANDS.len());
    println!("  Skill: .claude/skills/ddd/SKILL.md");
    println!("Restart Claude Code to use /ddd-<command> syntax");

    Ok(())
}

fn setup_opencode(ddd_binary: &Path, project_root: &Path) -> Result<()> {
    let commands_dir = project_root.join(".opencode/commands");

    // Create directories
    fs::create_dir_all(&commands_dir)?;

    // Backup existing command files
    backup_dir(&commands_dir, "ddd-", ".md")?;

    // Generate command files (10 files)
    for (name, desc) in PUBLIC_COMMANDS {
        let cmd_file = commands_dir.join(format!("ddd-{}.md", name));
        // init 命令需要 --context 参数
        let (arg_format, desc_suffix) = if *name == "init" {
            ("--context $ARGUMENTS", " (requires --context)")
        } else {
            ("$ARGUMENTS", "")
        };
        let content = format!(
            r#"---
description: "DocDriven CLI - {}{}"
---

!`{} {} {}`
"#,
            desc,
            desc_suffix,
            ddd_binary.to_string_lossy(),
            name,
            arg_format
        );
        fs::write(&cmd_file, content)?;
    }

    println!("OpenCode setup complete!");
    println!("  Commands: .opencode/commands/ddd-*.md ({} files)", PUBLIC_COMMANDS.len());
    println!("Restart OpenCode to use /ddd-<command> syntax");

    Ok(())
}

fn backup_dir(dir: &Path, prefix: &str, extension: &str) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) && name.ends_with(extension) {
            let backup_path = entry.path().with_extension("bak");
            fs::copy(entry.path(), backup_path)?;
            println!("Backed up: {}", name);
        }
    }
    Ok(())
}

fn backup_file(path: &Path) -> Result<()> {
    if path.exists() {
        let backup_path = path.with_extension(
            path.extension()
                .map(|e| format!("{}.bak", e.to_string_lossy()))
                .unwrap_or_else(|| "bak".to_string()),
        );
        fs::copy(path, backup_path)?;
        println!("Backed up: {}", path.file_name().unwrap().to_string_lossy());
    }
    Ok(())
}
