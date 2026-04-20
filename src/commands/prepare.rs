use crate::commands::{DddContext, PrepareCmd};
use crate::prompts::render;
use anyhow::Result;
use std::fs;

const PREPARE_PROMPT: &str = r#"根据 @project_docs/specs/ 下的spec, 启动规划子代理, 规划开发阶段, 生成每个阶段的任务清单以及要引用的spec文件列表(index是一定每一个都要引用的).以及该阶段结束需要验证的验证清单, 将开发计划拆分成 index + 每阶段文档的形式, 方便根据阶段名称精确找到对应的文档, 所有生成文件存到 @project_docs/phrases/ 下.
完成后调用 !`ddd-tool gen_phrase` 生成状态机."#;

pub fn run(_cmd: PrepareCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验状态
    let state = ctx.load_state()?;
    if state.workflow != "init" {
        println!("当前已进入开发阶段, 请先完成当前开发任务");
        return Ok(());
    }

    // 清空 phrases 目录
    let phrases_dir = ctx.project_root.join("project_docs").join("phrases");
    if phrases_dir.exists() {
        fs::remove_dir_all(&phrases_dir)?;
    }
    fs::create_dir_all(&phrases_dir)?;

    // 渲染 Prompt
    let prompt = render(
        PREPARE_PROMPT,
        &crate::prompts::PromptParams::new(),
    );

    println!("{}", prompt);

    Ok(())
}
