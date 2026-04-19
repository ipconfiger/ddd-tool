use crate::commands::{DddContext, InitCmd};
use crate::prompts::render;
use anyhow::Result;
use std::fs;

const INIT_PROMPT: &str = r#"调用 @Plan 分析需求 {context} 将需求拆分到模块后,单独为模块生成 .md 文件,然后按照 LLM Wiki的方式将拆分后模块的spec文件连接起来 , 所有文档存储到
@project_docs/specs/ 下."#;

pub fn run(cmd: InitCmd) {
    if let Err(e) = do_run(cmd) {
        eprintln!("错误: {}", e);
    }
}

fn do_run(cmd: InitCmd) -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验状态
    let state = ctx.load_state()?;
    if state.workflow != "init" {
        println!("当前已进入开发阶段, 请先完成当前开发任务");
        return Ok(());
    }

    // 读取 context 文档内容
    let context_value = if let Some(ref context_path) = cmd.context {
        let resolved = ctx.resolve_path(context_path);
        if resolved.exists() {
            fs::read_to_string(&resolved).unwrap_or_else(|_| context_path.clone())
        } else {
            context_path.clone()
        }
    } else {
        "未提供需求文档".to_string()
    };

    // 渲染 Prompt
    let prompt = render(
        INIT_PROMPT,
        &crate::prompts::PromptParams::new().with_context(context_value),
    );

    println!("{}", prompt);

    Ok(())
}
