use crate::commands::DddContext;
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::Result;
use std::ffi::OsStr;
use std::fs;

pub struct AcceptCommand;

impl DddCommand for AcceptCommand {
    fn name(&self) -> &'static str {
        "ddd-accept"
    }

    fn description(&self) -> &'static str {
        "Accept development plan, init phases"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        None
    }

    fn command_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "加载Skill {name}, 执行技能生成开发任务批次, 完成后提示用户可以执行 /ddd-exec 开始进行开发"
        ))
    }

    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "{name}"
description: "接受阶段计划并初始化开发状态"
---
扫描 phases/ 目录并初始化开发阶段
调用Bash !`{} accept 2>&1`
"#,
            bin
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let phases_dir = ctx.project_root.join("project_docs").join("phases");
        let mut phase_files: Vec<_> = fs::read_dir(&phases_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                name.to_string_lossy().ends_with(".md") && !name.to_string_lossy().starts_with("index")
            })
            .collect();

        phase_files.sort_by_cached_key(|e| extract_sort_key(&e.file_name()));

        let files: Vec<_> = phase_files
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let name = format!("Phase{}", idx + 1);
                let file = format!(
                    "@project_docs/phases/{}",
                    entry.file_name().to_string_lossy()
                );
                (name, file)
            })
            .collect();

        if files.is_empty() {
            return Ok(CommandResult::ok("显示:开发计划未生成, 请重新执行 /ddd-prepare, **important** 状态机由ddd-tool维护, 不允许修改 roadmap.json"));
        }

        let mut state = ctx.load_state()?;
        state.init_phases_from_files(files);

        ctx.save_state(&state)?;
        Ok(CommandResult::ok(format!("状态机已生成，共 {} 个阶段", state.phases.len())))
    }
}

/// 从文件名中提取第一个连续数字序列，用于自然排序
fn extract_sort_key(filename: &OsStr) -> (Option<u32>, String) {
    let s = filename.to_string_lossy();
    let num = s
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse::<u32>().ok());
    (num, s.into_owned())
}

/// accept: 扫描 phases 目录，生成 phases 数组
pub fn accept() -> Result<()> {
    let ctx = DddContext::new()?;

    // 扫描 phases 目录（排除 index.md）
    let phases_dir = ctx.project_root.join("project_docs").join("phases");
    let mut phase_files: Vec<_> = fs::read_dir(&phases_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            name.to_string_lossy().ends_with(".md") && !name.to_string_lossy().starts_with("index")
        })
        .collect();

    phase_files.sort_by_cached_key(|e| extract_sort_key(&e.file_name()));

    // 构建 phases 初始化数据
    let files: Vec<_> = phase_files
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let name = format!("Phase{}", idx + 1);
            let file = format!(
                "@project_docs/phases/{}",
                entry.file_name().to_string_lossy()
            );
            (name, file)
        })
        .collect();

    if files.is_empty() {
        println!("显示:开发计划未生成, 请重新执行 /ddd-prepare, **important** 状态机由ddd-tool维护, 不允许修改 roadmap.json");
        return Ok(());
    }

    // 更新状态
    let mut state = ctx.load_state()?;
    state.init_phases_from_files(files);

    ctx.save_state(&state)?;

    println!("状态机已生成，共 {} 个阶段, 提示: 请执行 /ddd-exec 开始启动实际开发, 然后停止!", state.phases.len());

    Ok(())
}
