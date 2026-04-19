# Setup Command Implementation Plan

## Requirements Summary

The `setup` command initializes project-level command/skill registration for Claude Code or OpenCode, enabling `ddd` subcommands to be invoked via `/ddd-xxx` (Claude) or `!ddd xxx` (OpenCode) syntax.

### Core Behavior
- `--tool claude`: Creates `.claude/commands/ddd-*.md` (10 files) + `.claude/skills/ddd/SKILL.md`
- `--tool opencode`: Creates `.opencode/skills/ddd/SKILL.md` + `ddd` wrapper in PATH

### Public Commands to Register
| Command | Description |
|---------|-------------|
| `init` | Initialize project with context |
| `prepare` | Prepare phrases from specs |
| `exec` | Execute development phase |
| `verify` | Verify phase成果 |
| `fix-plan` | Generate fix plan |
| `fix-exec` | Execute fix plan |
| `archive` | Archive completed project |
| `report` | Generate project report |
| `sync` | Sync code to docs |
| `resume` | Resume interrupted workflow |

**Note:** Internal commands (`gen-phrase`, `set-issue`, `finish-fix`, `finish-phrase`) are NOT registered as they are called via `!ddd` from prompts internally.

---

## Architecture

```
Claude Code:
.claude/
├── commands/
│   ├── ddd-init.md      → invokes skill with "init $ARGUMENTS"
│   ├── ddd-prepare.md
│   ├── ddd-exec.md
│   ├── ddd-verify.md
│   ├── ddd-fix-plan.md
│   ├── ddd-fix-exec.md
│   ├── ddd-archive.md
│   ├── ddd-report.md
│   ├── ddd-sync.md
│   └── ddd-resume.md
└── skills/
    └── ddd/
        └── SKILL.md    → main skill entry point

OpenCode:
.opencode/
├── skills/
│   └── ddd/
│       ├── SKILL.md    → skill documentation
│       └── bin/
            └── ddd     → wrapper script for !ddd invocation
```

---

## Implementation Steps

### 1. Add SetupCmd to Command Enum
**File:** `src/commands/mod.rs`

```rust
#[derive(Parser, Debug)]
pub struct SetupCmd {
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(["claude", "opencode"]),
           help = "Target tool: claude or opencode")]
    pub tool: String,
}
```

Add to Command enum:
```rust
Setup(SetupCmd),
```

Add dispatch:
```rust
Command::Setup(c) => setup::run(c),
```

### 2. Create `src/commands/setup.rs`

```rust
use crate::commands::DddContext;
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

pub fn run(cmd: SetupCmd) {
    let ctx = DddContext::new().expect("Failed to create context");
    let ddd_binary = std::env::current_exe()
        .expect("Failed to get current exe");

    match cmd.tool.as_str() {
        "claude" => setup_claude(&ddd_binary, &ctx.project_root),
        "opencode" => setup_opencode(&ddd_binary, &ctx.project_root),
        _ => println!("Invalid tool: {}. Use 'claude' or 'opencode'", cmd.tool),
    }
}
```

### 3. Implement Claude Setup

**Claude commands** are markdown files that invoke skills:
```markdown
---
description: "DocDriven CLI - init command"
---

Invoke the ddd skill with "init $ARGUMENTS"
```

**Claude skill** is the main entry point that dispatches to CLI:
```markdown
---
name: ddd
description: DocDriven CLI - 文档驱动开发框架
---

# DocDriven CLI

## Usage
Use `/ddd-<command>` to invoke commands.
```

```rust
fn setup_claude(ddd_binary: &Path, project_root: &Path) -> Result<()> {
    let claude_dir = project_root.join(".claude");
    let commands_dir = claude_dir.join("commands");
    let skills_dir = claude_dir.join("skills").join("ddd");

    // Create directories
    fs::create_dir_all(&commands_dir)?;
    fs::create_dir_all(&skills_dir)?;

    // Backup existing files if any
    backup_existing(&commands_dir, "ddd-*.md")?;
    backup_existing(&skills_dir, "SKILL.md")?;

    // Generate command files (10 files)
    for (name, desc) in PUBLIC_COMMANDS {
        let cmd_file = commands_dir.join(format!("ddd-{}.md", name));
        let content = format!(r#"---
description: "DocDriven CLI - {}"
---

Invoke the ddd skill with "{} $ARGUMENTS"
"#,
            desc, name);
        fs::write(&cmd_file, content)?;
    }

    // Generate main skill file
    let skill_content = format!(r#"---
name: ddd
description: DocDriven CLI - 文档驱动开发框架
---

# DocDriven CLI

## Available Commands

{}

## Usage

Use `/ddd-<command>` to invoke any command.

"#, PUBLIC_COMMANDS
        .iter()
        .map(|(name, desc)| format!("- `/ddd-{}`: {}", name, desc))
        .collect::<Vec<_>>()
        .join("\n"));

    fs::write(skills_dir.join("SKILL.md"), skill_content)?;

    println!("Claude Code setup complete!");
    println!("  Commands: .claude/commands/ddd-*.md ({} files)", PUBLIC_COMMANDS.len());
    println!("  Skill: .claude/skills/ddd/SKILL.md");
    println!("Restart Claude Code to use /ddd-<command> syntax");

    Ok(())
}
```

### 4. Implement OpenCode Setup

OpenCode uses `!ddd <cmd>` syntax which calls CLI directly if `ddd` is in PATH. We create:
1. A SKILL.md for skill documentation
2. A wrapper script that gets added to PATH

```rust
fn setup_opencode(ddd_binary: &Path, project_root: &Path) -> Result<()> {
    let skills_dir = project_root.join(".opencode/skills/ddd");
    let bin_dir = skills_dir.join("bin");

    // Create directories
    fs::create_dir_all(&bin_dir)?;

    // Backup existing
    backup_existing(&skills_dir, "SKILL.md")?;
    backup_existing(&bin_dir, "ddd")?;

    // Generate SKILL.md
    let skill_content = format!(r#"---
name: ddd
description: DocDriven CLI - 文档驱动开发框架
---

# DocDriven CLI

## Available Commands

{}

## Usage

1. Add to PATH: export PATH="$PATH:$(pwd)/.opencode/skills/ddd/bin"
2. Use `!ddd <command>` to invoke commands.

"#, PUBLIC_COMMANDS
        .iter()
        .map(|(name, desc)| format!("- `ddd {}`: {}", name, desc))
        .collect::<Vec<_>>()
        .join("\n"));

    fs::write(skills_dir.join("SKILL.md"), skill_content)?;

    // Generate wrapper script
    let wrapper = format!(r#"#!/bin/bash
# DocDriven CLI wrapper for OpenCode
# Usage: !ddd <command>

DDD_BIN="{}"
COMMAND="$1"

if [ -z "$COMMAND" ]; then
    echo "Usage: ddd <command>"
    echo "Available: {}"
    exit 1
fi

"$DDD_BIN" "$COMMAND" "${@:2}"
"#,
        ddd_binary.to_string_lossy(),
        PUBLIC_COMMANDS.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", "));

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
```

### 5. Backup Helper

```rust
fn backup_existing(dir: &Path, pattern: &str) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy();
        if name.contains("ddd") || name == "SKILL.md" {
            let backup_path = entry.path().with_extension("bak");
            fs::copy(entry.path(), backup_path)?;
            println!("Backed up: {}", name);
        }
    }
    Ok(())
}
```

### 6. Register Module
**File:** `src/commands/mod.rs`

```rust
pub mod setup;
```

---

## Data Flow

```
User: ddd setup --tool claude
  │
  ├─► SetupCmd { tool: "claude" }
  │
  ├─► setup::run(cmd)
  │     ├─► get current_exe() → /path/to/ddd
  │     │
  │     └─► setup_claude(ddd_binary, project_root)
  │           ├─► .claude/commands/ddd-init.md
  │           ├─► .claude/commands/ddd-prepare.md
  │           ├─► ... (10 files total)
  │           └─► .claude/skills/ddd/SKILL.md
  │
  └─► Success message

User: ddd setup --tool opencode
  │
  └─► setup_opencode()
        ├─► .opencode/skills/ddd/SKILL.md
        └─► .opencode/skills/ddd/bin/ddd (wrapper)
```

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Invalid `--tool` value | Print error with valid options |
| Binary path resolution fails | Print error, exit with code 1 |
| File write permission denied | Print error with path, exit with code 1 |
| Existing config file | Backup to `.bak` before overwrite |

---

## Verification Steps

1. **Build**: `cargo build` succeeds with 0 warnings
2. **Tests**: `cargo test` passes
3. **Help**: `ddd setup --help` shows correct usage
4. **Claude setup**:
   ```bash
   cd /tmp && mkdir test_claude && cd test_claude
   ddd setup --tool claude
   ls -la .claude/commands/  # Should have 10 ddd-*.md files
   cat .claude/skills/ddd/SKILL.md  # Should have frontmatter + command list
   ```
5. **OpenCode setup**:
   ```bash
   cd /tmp && mkdir test_opencode && cd test_opencode
   ddd setup --tool opencode
   cat .opencode/skills/ddd/SKILL.md  # Should have frontmatter
   ls -la .opencode/skills/ddd/bin/ddd  # Should be executable
   ```

---

## File Reference

| File | Change |
|------|--------|
| `src/commands/mod.rs` | Add SetupCmd struct, Command::Setup variant, dispatch, `pub mod setup` |
| `src/commands/setup.rs` | New file with `setup_claude()`, `setup_opencode()`, `backup_existing()` |
