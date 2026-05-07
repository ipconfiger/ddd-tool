use crate::commands::DddContext;
use anyhow::Result;
use std::ffi::OsStr;
use std::fs;

/// 从文件名中提取第一个连续数字序列，用于自然排序
fn extract_sort_key(filename: &OsStr) -> (Option<u32>, String) {
    let s = filename.to_string_lossy();
    let num = s
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse::<u32>().ok());
    (num, s.into_owned())
}

/// accept: 扫描 phrases 目录，生成 phrases 数组
pub fn accept() -> Result<()> {
    let ctx = DddContext::new()?;

    // 扫描 phrases 目录（排除 index.md）
    let phrases_dir = ctx.project_root.join("project_docs").join("phases");
    let mut phrase_files: Vec<_> = fs::read_dir(&phrases_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            name.to_string_lossy().ends_with(".md") && !name.to_string_lossy().starts_with("index")
        })
        .collect();

    phrase_files.sort_by_cached_key(|e| extract_sort_key(&e.file_name()));

    // 构建 phrases 初始化数据
    let files: Vec<_> = phrase_files
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let name = format!("Phrase{}", idx);
            let file = format!(
                "@project_docs/phases/{}",
                entry.file_name().to_string_lossy()
            );
            (name, file)
        })
        .collect();

    if files.is_empty() {
        println!("显示:开发计划未生成, 请重新执行 /ddd-prepare, 停止执行!");
        return Ok(());
    }

    // 更新状态
    let mut state = ctx.load_state()?;
    state.init_phrases_from_files(files);

    ctx.save_state(&state)?;

    println!("状态机已生成，共 {} 个阶段", state.phrases.len());

    Ok(())
}
