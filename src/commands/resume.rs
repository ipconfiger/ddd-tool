use crate::commands::{DddContext, ResumeCmd};
use anyhow::Result;

pub fn run(_cmd: ResumeCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    let state = ctx.load_state()?;

    // 扫描 phrases 查找断点
    let in_progress_phase = state.phrases.iter().find(|p| {
        p.status == "dev" || p.status == "issue_found" || p.status == "fixing"
    });

    // 扫描 fixes 查找 executing
    let in_progress_fix = state.phrases.iter()
        .flat_map(|p| p.fixes.iter())
        .find(|f| f.status == "executing");

    let _current_phase = state.current_phase.as_ref();

    if let Some(phase) = in_progress_phase {
        println!("📍 发现断点:");
        println!("  当前阶段: {}", phase.name);
        println!("  阶段状态: {}", phase.status);
        println!("  文件: {}", phase.file);

        if let Some(fix) = in_progress_fix {
            println!("  修复任务: {} (executing)", fix.plan_file);
        }

        println!();
        println!("请选择下一步操作:");
        println!("  /ddd-exec - 继续当前阶段开发");
        println!("  /ddd-verify - 审核当前阶段成果");
    } else {
        println!("没有发现断点，所有阶段已完成。");
        println!("当前工作流状态: {}", state.workflow);
    }

    Ok(())
}
