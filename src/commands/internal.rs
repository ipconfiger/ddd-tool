use crate::commands::DddContext;
use crate::state::Phrase;
use anyhow::Result;
use std::fs;

/// gen-phrase: 扫描 phrases 目录，生成 phrases 数组
pub fn gen_phrase() -> Result<()> {
    let ctx = DddContext::new()?;

    // 扫描 phrases 目录（排除 index.md）
    let phrases_dir = ctx.project_root.join("project_docs").join("phrases");
    let mut phrase_files: Vec<_> = fs::read_dir(&phrases_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            name.to_string_lossy().ends_with(".md") && !name.to_string_lossy().starts_with("index")
        })
        .collect();

    phrase_files.sort_by_key(|e| e.file_name());

    // 生成 phrases 数组
    let phrases: Vec<_> = phrase_files
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let name = format!("Phrase{}", idx);
            let file = format!(
                "@project_docs/phrases/{}",
                entry.file_name().to_string_lossy()
            );
            serde_json::json!({
                "name": name,
                "status": "init",
                "file": file,
                "fixes": []
            })
        })
        .collect();

    // 更新状态
    let mut state = ctx.load_state()?;
    state.doc_ready = true;
    state.workflow = "ready".to_string();
    state.current_phase = phrases.first().and_then(|p| p.get("name")).map(|n| n.as_str().unwrap().to_string());
    state.phrases = phrases
        .into_iter()
        .map(|p| {
            Phrase {
                name: p["name"].as_str().unwrap().to_string(),
                status: p["status"].as_str().unwrap().to_string(),
                file: p["file"].as_str().unwrap().to_string(),
                fixes: vec![],
            }
        })
        .collect();

    ctx.save_state(&state)?;

    println!("状态机已生成，共 {} 个阶段", state.phrases.len());

    Ok(())
}

/// set-issue: 设置当前 phase 状态为 issue_found
pub fn set_issue() -> Result<()> {
    let ctx = DddContext::new()?;
    let mut state = ctx.load_state()?;

    if let Some(ref current) = state.current_phase {
        if let Some(phase) = state.phrases.iter_mut().find(|p| &p.name == current) {
            phase.status = "issue_found".to_string();
            ctx.save_state(&state)?;
            println!("issue已记录, 请执行 /ddd-fix-plan 来生成修复计划");
            return Ok(());
        }
    }

    println!("没有正在进行的阶段");
    Ok(())
}

/// finish-fix: 完成修复
pub fn finish_fix() -> Result<()> {
    let ctx = DddContext::new()?;
    let mut state = ctx.load_state()?;

    if let Some(ref current) = state.current_phase {
        if let Some(phase) = state.phrases.iter_mut().find(|p| &p.name == current) {
            // 找到 executing 或最后一个 fix
            if let Some(fix) = phase.fixes.iter_mut().find(|f| f.status == "executing") {
                fix.status = "done".to_string();
            } else if let Some(fix) = phase.fixes.last_mut() {
                fix.status = "done".to_string();
            }

            // 检查是否所有 fixes 都 done
            let all_done = phase.fixes.iter().all(|f| f.status == "done");
            if all_done {
                phase.status = "fixing".to_string();
            }

            ctx.save_state(&state)?;
            println!("是否要执行 /ddd-exec 开始下一个阶段的开发");
            return Ok(());
        }
    }

    println!("没有正在进行的阶段");
    Ok(())
}

/// finish-phrase: 完成阶段
pub fn finish_phrase() -> Result<()> {
    let ctx = DddContext::new()?;
    let mut state = ctx.load_state()?;

    if let Some(ref current) = state.current_phase {
        if let Some(phase) = state.phrases.iter_mut().find(|p| &p.name == current) {
            phase.status = "finished".to_string();
            ctx.save_state(&state)?;
            println!("是否要执行 /ddd-exec 开始下一个阶段的开发");
            return Ok(());
        }
    }

    println!("没有正在进行的阶段");
    Ok(())
}
