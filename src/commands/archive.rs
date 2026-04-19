use crate::commands::{DddContext, ArchiveCmd};
use anyhow::Result;
use std::fs;

pub fn run(_cmd: ArchiveCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    let state = ctx.load_state()?;

    // 校验所有 phrases 已完成
    let unfinished: Vec<_> = state.phrases.iter()
        .filter(|p| p.status != "finished")
        .collect();

    if !unfinished.is_empty() {
        println!("请先完成所有开发阶段:");
        for p in &unfinished {
            println!("  - {} (状态: {})", p.name, p.status);
        }
        return Ok(());
    }

    // 计算归档路径
    let archives_dir = ctx.project_root.join("project_docs").join("archives");
    fs::create_dir_all(&archives_dir)?;

    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let idx = fs::read_dir(&archives_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            e.path().file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&today))
                .unwrap_or(false)
        })
        .count();

    let archive_name = format!("{}-{}", today, idx);
    let archive_path = archives_dir.join(&archive_name);
    fs::create_dir_all(&archive_path)?;

    // 移动相关文档
    let docs_to_archive = vec![
        ("specs", "specs"),
        ("phrases", "phrases"),
    ];

    for (src_name, _dst_name) in docs_to_archive {
        let src = ctx.project_root.join("project_docs").join(src_name);
        let dst = archive_path.join(src_name);
        if src.exists() {
            fs::rename(&src, &dst)?;
        }
    }

    // 重置 roadmap.json
    let initial_state = crate::state::RoadmapState::new();
    ctx.save_state(&initial_state)?;

    println!("✅ 项目已归档到: @project_docs/archives/{}/", archive_name);
    println!("roadmap.json 已重置为初始状态。");

    Ok(())
}
