use crate::commands::{DddContext, ArchiveCmd};
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::Path;
use tar::Builder;

pub struct ArchiveCommand;

impl DddCommand for ArchiveCommand {
    fn name(&self) -> &'static str {
        "archive"
    }

    fn description(&self) -> &'static str {
        "Archive completed project"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        None
    }

    fn command_prompt(&self, bin: &str) -> Option<String> {
        Some(format!(
            "使用 Bash工具 执行: {} archive。归档已完成的项目。将规格文档和阶段计划打包为 tar.gz 存档到 @project_docs/archives/ 目录。归档后项目状态重置, 可开始新项目。",
            bin
        ))
    }

    fn skill_prompt(&self, bin: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "archive"
description: "归档已完成项目, 打包规格和阶段文档"
---
调用 !`{} archive 2>&1`
打包归档项目文档到 archives/ 目录
"#,
            bin
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let state = ctx.load_state()?;

        // 1. 校验所有 phases 已完成
        let unfinished: Vec<_> = state.phases.iter()
            .filter(|p| p.status != "finished")
            .collect();
        if !unfinished.is_empty() {
            let mut msg = String::from("请先完成所有开发阶段:\n");
            for p in &unfinished {
                msg.push_str(&format!("  - {} (状态: {})\n", p.name, p.status));
            }
            return Ok(CommandResult::ok(msg));
        }

        // 2. 创建归档目录
        let project_docs = ctx.project_root.join("project_docs");
        let archives_dir = project_docs.join("archives");
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

        // 3. gzip 归档 specs 与 phases 到单一 tar.gz
        let tar_gz_name = format!("archive-{}.tar.gz", archive_name);
        let tar_gz_path = archive_path.join(&tar_gz_name);
        let sources: Vec<(&str, std::path::PathBuf)> = vec![
            ("specs", project_docs.join("specs")),
            ("phases", project_docs.join("phases")),
        ];
        archive_dirs(&sources, &tar_gz_path)
            .with_context(|| format!("归档失败: {}", tar_gz_path.display()))?;

        // 4. 清空 specs/ 与 phases/
        for (_name, src) in &sources {
            if src.exists() {
                fs::remove_dir_all(src)?;
            }
            fs::create_dir_all(src)?;
        }

        // 5. 重置 roadmap.json
        let initial_state = crate::state::RoadmapState::new();
        ctx.save_state(&initial_state)?;

        let msg = format!("✅ 项目已归档到: @project_docs/archives/{}/\n  - {}\nroadmap.json 已重置为初始状态。", archive_name, tar_gz_name);
        Ok(CommandResult::ok(msg))
    }
}

pub fn run(_cmd: ArchiveCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    let state = ctx.load_state()?;

    // 1. 校验所有 phases 已完成
    let unfinished: Vec<_> = state.phases.iter()
        .filter(|p| p.status != "finished")
        .collect();
    if !unfinished.is_empty() {
        println!("请先完成所有开发阶段:");
        for p in &unfinished {
            println!("  - {} (状态: {})", p.name, p.status);
        }
        return Ok(());
    }

    // 2. 创建归档目录
    let project_docs = ctx.project_root.join("project_docs");
    let archives_dir = project_docs.join("archives");
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

    // 3. gzip 归档 specs 与 phases 到单一 tar.gz
    let tar_gz_name = format!("archive-{}.tar.gz", archive_name);
    let tar_gz_path = archive_path.join(&tar_gz_name);
    let sources: Vec<(&str, std::path::PathBuf)> = vec![
        ("specs", project_docs.join("specs")),
        ("phases", project_docs.join("phases")),
    ];
    archive_dirs(&sources, &tar_gz_path)
        .with_context(|| format!("归档失败: {}", tar_gz_path.display()))?;

    // 4. 清空 specs/ 与 phases/
    for (_name, src) in &sources {
        if src.exists() {
            fs::remove_dir_all(src)?;
        }
        fs::create_dir_all(src)?;
    }

    // 5. 重置 roadmap.json
    let initial_state = crate::state::RoadmapState::new();
    ctx.save_state(&initial_state)?;

    println!("✅ 项目已归档到: @project_docs/archives/{}/", archive_name);
    println!("  - {}", tar_gz_name);
    println!("roadmap.json 已重置为初始状态。");
    Ok(())
}

/// 将多个源目录打包为单个 .tar.gz；每个源在归档内以其第一个元素作为目录前缀。
/// 源目录不存在则跳过。
pub(crate) fn archive_dirs(sources: &[(&str, std::path::PathBuf)], dst: &Path) -> Result<()> {
    let file = File::create(dst)
        .with_context(|| format!("创建归档文件失败: {}", dst.display()))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

    for (name, src) in sources {
        if !src.exists() {
            continue;
        }
        builder
            .append_dir_all(name, src)
            .with_context(|| format!("打包目录失败: {} -> /{}", src.display(), name))?;
    }

    let encoder = builder.into_inner().context("关闭 gzip 编码器失败")?;
    encoder.finish().context("关闭归档文件失败")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tar::Archive;
    use tempfile::tempdir;

    /// 读取 tar 归档内所有文件路径
    fn list_tar_entries(tar_gz: &Path) -> Vec<String> {
        let file = File::open(tar_gz).unwrap();
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive
            .entries()
            .unwrap()
            .map(|e| e.unwrap().path().unwrap().to_string_lossy().to_string())
            .collect()
    }

    /// 读取 tar 归档内指定路径的文件内容
    fn read_tar_entry(tar_gz: &Path, target: &str) -> String {
        let file = File::open(tar_gz).unwrap();
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        let mut entry = archive
            .entries()
            .unwrap()
            .find(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .unwrap()
                    .to_string_lossy()
                    == target
            })
            .expect("entry not found")
            .unwrap();
        let mut s = String::new();
        entry.read_to_string(&mut s).unwrap();
        s
    }

    #[test]
    fn test_archive_dirs_creates_tar_gz_file() {
        let tmp = tempdir().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("spec1.md"), "spec content").unwrap();

        let dst = tmp.path().join("out.tar.gz");
        archive_dirs(&[("specs", specs.clone())], &dst).unwrap();

        assert!(dst.exists());
        let meta = fs::metadata(&dst).unwrap();
        assert!(meta.len() > 0, "归档文件应非空");
    }

    #[test]
    fn test_archive_dirs_contains_specs_prefix() {
        let tmp = tempdir().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("spec1.md"), "spec content").unwrap();

        let dst = tmp.path().join("out.tar.gz");
        archive_dirs(&[("specs", specs)], &dst).unwrap();

        let entries = list_tar_entries(&dst);
        assert!(entries.iter().any(|e| e.starts_with("specs/")),
            "归档内应含 specs/ 前缀条目，实际: {:?}", entries);
    }

    #[test]
    fn test_archive_dirs_contains_phases_prefix() {
        let tmp = tempdir().unwrap();
        let phases = tmp.path().join("phases");
        fs::create_dir_all(&phases).unwrap();
        fs::write(phases.join("1_foo.md"), "phase content").unwrap();

        let dst = tmp.path().join("out.tar.gz");
        archive_dirs(&[("phases", phases)], &dst).unwrap();

        let entries = list_tar_entries(&dst);
        assert!(entries.iter().any(|e| e.starts_with("phases/")));
    }

    #[test]
    fn test_archive_dirs_combines_multiple_sources() {
        let tmp = tempdir().unwrap();
        let specs = tmp.path().join("specs");
        let phases = tmp.path().join("phases");
        fs::create_dir_all(&specs).unwrap();
        fs::create_dir_all(&phases).unwrap();
        fs::write(specs.join("s.md"), "S").unwrap();
        fs::write(phases.join("p.md"), "P").unwrap();

        let dst = tmp.path().join("out.tar.gz");
        archive_dirs(
            &[("specs", specs.clone()), ("phases", phases.clone())],
            &dst,
        ).unwrap();

        let entries = list_tar_entries(&dst);
        assert!(entries.iter().any(|e| e.starts_with("specs/")));
        assert!(entries.iter().any(|e| e.starts_with("phases/")));
    }

    #[test]
    fn test_archive_dirs_preserves_file_content() {
        let tmp = tempdir().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        let original = "# spec\nline2\nline3\n";
        fs::write(specs.join("s.md"), original).unwrap();

        let dst = tmp.path().join("out.tar.gz");
        archive_dirs(&[("specs", specs)], &dst).unwrap();

        let restored = read_tar_entry(&dst, "specs/s.md");
        assert_eq!(restored, original);
    }

    #[test]
    fn test_archive_dirs_skips_missing_source() {
        let tmp = tempdir().unwrap();
        let missing = tmp.path().join("nonexistent");
        let dst = tmp.path().join("out.tar.gz");

        // 不存在的源目录不报错
        archive_dirs(&[("missing", missing)], &dst).unwrap();
        assert!(dst.exists());
    }

    #[test]
    fn test_archive_dirs_handles_empty_dir() {
        let tmp = tempdir().unwrap();
        let empty = tmp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();

        let dst = tmp.path().join("out.tar.gz");
        archive_dirs(&[("empty", empty)], &dst).unwrap();
        assert!(dst.exists());
    }
}
