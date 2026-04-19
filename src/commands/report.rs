use crate::commands::{DddContext, ReportCmd};
use anyhow::Result;
use std::fs;

pub fn run(_cmd: ReportCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    let state = ctx.load_state()?;

    let report_path = ctx.project_root.join("project_docs").join("report.md");

    // 生成报告
    let report = generate_report(&state);

    fs::write(&report_path, &report)?;

    println!("📊 报告已生成: @project_docs/report.md");
    println!();
    println!("{}", report);

    Ok(())
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

    for phrase in &state.phrases {
        report.push_str(&format!("| {} | {} | {} |\n", phrase.name, phrase.status, phrase.file));
    }

    report.push_str("\n## 状态流转图\n\n");
    report.push_str("```\n");
    report.push_str(&format!("workflow: {} → ", state.workflow));
    if let Some(cp) = &state.current_phase {
        report.push_str(&format!("current_phase: {}", cp));
    } else {
        report.push_str("current_phase: null");
    }
    report.push_str("\n```\n");

    // 缺陷统计
    let total_fixes: usize = state.phrases.iter().map(|p| p.fixes.len()).sum();
    let done_fixes: usize = state.phrases.iter()
        .flat_map(|p| p.fixes.iter())
        .filter(|f| f.status == "done")
        .count();

    report.push_str("\n## 缺陷统计\n\n");
    report.push_str(&format!("- 总修复任务: {}\n", total_fixes));
    report.push_str(&format!("- 已完成: {}\n", done_fixes));
    report.push_str(&format!("- 闭环率: {}%\n",
        if total_fixes > 0 { done_fixes * 100 / total_fixes } else { 100 }));

    report.push_str("\n---\n\n*报告由 DocDriven CLI 自动生成*\n");

    report
}
