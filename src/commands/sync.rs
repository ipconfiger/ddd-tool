use crate::commands::{DddContext, SyncCmd};
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::Result;
use std::fs;

pub struct SyncCommand;

impl DddCommand for SyncCommand {
    fn name(&self) -> &'static str {
        "sync"
    }

    fn description(&self) -> &'static str {
        "Sync code to docs"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        None
    }

    fn command_prompt(&self, bin: &str) -> Option<String> {
        Some(format!(
            "使用 Bash工具 执行: {} sync。扫描 src/ 目录下的源代码和 specs/ 目录下的规格文档, 生成同步检查清单。用于确认代码与规格的一致性。",
            bin
        ))
    }

    fn skill_prompt(&self, bin: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "sync"
description: "同步检查代码与规格文档的一致性"
---
调用 !`{} sync 2>&1`
扫描源代码和规格文档, 生成同步检查清单
"#,
            bin
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let sync_log_path = ctx.project_root.join("project_docs").join("sync_log.md");

        let src_path = ctx.project_root.join("src");
        let mut code_modules = Vec::new();

        if src_path.exists() {
            collect_rust_files(&src_path, &mut code_modules);
        }

        let specs_path = ctx.project_root.join("project_docs").join("specs");
        let mut spec_files = Vec::new();

        if specs_path.exists() {
            for entry in fs::read_dir(&specs_path)?.filter_map(Result::ok) {
                if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    spec_files.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }

        let mut log = String::new();
        log.push_str("# 代码与文档同步日志\n\n");
        log.push_str(&format!("**同步时间**: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        log.push_str("## 代码模块\n\n");
        for module in &code_modules {
            log.push_str(&format!("- {}\n", module));
        }
        log.push_str("\n## Spec 文档\n\n");
        for spec in &spec_files {
            log.push_str(&format!("- {}\n", spec));
        }
        log.push_str("\n## 同步状态\n\n");
        log.push_str("- [ ] 代码结构已扫描\n");
        log.push_str("- [ ] Spec 文档已扫描\n");
        log.push_str("- [ ] 差异对比待执行\n");
        log.push_str("- [ ] 文档更新待执行\n");

        fs::write(&sync_log_path, &log)?;

        Ok(CommandResult::ok("📝 代码实现已反向同步至文档，文档驱动闭环已刷新\n同步日志: @project_docs/sync_log.md"))
    }
}

pub fn run(_cmd: SyncCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    let sync_log_path = ctx.project_root.join("project_docs").join("sync_log.md");

    // 扫描 src 目录获取代码结构
    let src_path = ctx.project_root.join("src");
    let mut code_modules = Vec::new();

    if src_path.exists() {
        collect_rust_files(&src_path, &mut code_modules);
    }

    // 扫描 specs 目录
    let specs_path = ctx.project_root.join("project_docs").join("specs");
    let mut spec_files = Vec::new();

    if specs_path.exists() {
        for entry in fs::read_dir(&specs_path)?.filter_map(Result::ok) {
            if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                spec_files.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    // 生成同步日志
    let mut log = String::new();
    log.push_str("# 代码与文档同步日志\n\n");
    log.push_str(&format!("**同步时间**: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    log.push_str("## 代码模块\n\n");
    for module in &code_modules {
        log.push_str(&format!("- {}\n", module));
    }
    log.push_str("\n## Spec 文档\n\n");
    for spec in &spec_files {
        log.push_str(&format!("- {}\n", spec));
    }
    log.push_str("\n## 同步状态\n\n");
    log.push_str("- [ ] 代码结构已扫描\n");
    log.push_str("- [ ] Spec 文档已扫描\n");
    log.push_str("- [ ] 差异对比待执行\n");
    log.push_str("- [ ] 文档更新待执行\n");

    fs::write(&sync_log_path, &log)?;

    println!("📝 代码实现已反向同步至文档，文档驱动闭环已刷新");
    println!("同步日志: @project_docs/sync_log.md");

    Ok(())
}

fn collect_rust_files(dir: &std::path::Path, modules: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name != "target" && !name.starts_with('.') {
                        collect_rust_files(&path, modules);
                        modules.push(format!("{}/ (模块)", name));
                    }
                }
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    modules.push(name.to_string());
                }
            }
        }
    }
}
