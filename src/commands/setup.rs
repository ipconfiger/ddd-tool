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
    let skills_dir = project_root.join(".opencode/skills/ddd");
    let bin_dir = skills_dir.join("bin");

    // Create directories
    fs::create_dir_all(&bin_dir)?;

    // Backup existing
    backup_file(skills_dir.join("SKILL.md").as_path())?;
    backup_file(bin_dir.join("ddd").as_path())?;

    // Generate SKILL.md
    let bin_dir_str = bin_dir.to_string_lossy();
    let skill_content = format!(
        r#"---
name: ddd
description: DocDriven CLI - 文档驱动开发框架
---

# DocDriven CLI

## Available Commands

{}

## Usage

1. Add to PATH: export PATH="$PATH:{}"
2. Use `!ddd <command>` to invoke commands.

"#,
        PUBLIC_COMMANDS
            .iter()
            .map(|(name, desc)| format!("- `ddd {}`: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n"),
        bin_dir_str
    );

    fs::write(skills_dir.join("SKILL.md"), skill_content)?;

    // Generate wrapper script
    let wrapper = format!(
        r#"#!/bin/bash
# DocDriven CLI wrapper for OpenCode
# Usage: !ddd <command>

DDD_BIN="{}"
COMMAND="$1"

if [ -z "$COMMAND" ]; then
    echo "Usage: ddd <command>"
    echo "Available: {}"
    exit 1
fi

"$DDD_BIN" "$COMMAND" "${{@:2}}"
"#,
        ddd_binary.to_string_lossy(),
        PUBLIC_COMMANDS
            .iter()
            .map(|(n, _)| *n)
            .collect::<Vec<_>>()
            .join(", ")
    );

    fs::write(bin_dir.join("ddd"), wrapper)?;

    // Make executable
    std::process::Command::new("chmod")
        .args(["+x", bin_dir.join("ddd").to_str().unwrap()])
        .output()?;

    println!("OpenCode setup complete!");
    println!("  Skill: .opencode/skills/ddd/SKILL.md");
    println!("  Wrapper: .opencode/skills/ddd/bin/ddd");
    println!();
    println!("Add to PATH: export PATH=\"$PATH:{}\"", bin_dir.to_string_lossy());
    println!("Then use: !ddd <command>");

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
