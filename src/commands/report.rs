use crate::commands::DddContext;
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::Result;

pub struct ReportCommand;

impl DddCommand for ReportCommand {
    fn name(&self) -> &'static str {
        "report"
    }

    fn description(&self) -> &'static str {
        "Generate project report"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        None
    }

    fn command_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "加载 Skill ddd-{name}, 执行技能",
        ))
    }

    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "ddd-{name}"
description: "生成项目开发进度报告"
---
调用 !`{} {name} 2>&1` 获取返回报告内容
直接提示报告内容
"#,
            bin
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let state = ctx.load_state()?;
        let report = generate_report(&state);
        Ok(CommandResult::ok(report))
    }
}

fn generate_report(state: &crate::state::RoadmapState) -> String {
    let mut report = String::new();

    report.push_str("# DocDriven 项目报告\n\n");
    report.push_str(&format!("**生成时间**: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    report.push_str(&format!("**工作流状态**: {}\n\n", state.workflow));

    // 阶段进度
    report.push_str("## 阶段进度\n\n");
    report.push_str("| 阶段 | 状态 | 文件 |\n");
    report.push_str("|------|------|------|\n");

    for phase in &state.phases {
        report.push_str(&format!("| {} | {} | {} |\n", phase.name, phase.status, phase.file));
    }

    report.push_str("\n---\n\n*报告由 DocDriven CLI 自动生成*\n");

    report
}
