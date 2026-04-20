use crate::commands::{DddContext, VerifyCmd};
use crate::prompts::render;
use anyhow::Result;

const VERIFY_PROMPT: &str = r#"根据开发计划: @{file} ,并从开发计划中提取对应的 spec 文档作为资料,然后
1. 对第一阶段开发进行代码审核.
2. 运行所有单元测试
3. 核对spec对代码进行深度事实审核
如果所有验证项目均没有issuse, 就执行 !`ddd-tool finish_phrase` 然后 输出 "太开心啦, 通过啦!".
如果有issuse, 就执行 !`ddd-tool set-issuse`."#;

pub fn run(_cmd: VerifyCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验状态
    let state = ctx.load_state()?;

    let current_phase = state.current_phase.as_ref()
        .and_then(|name| state.phrases.iter().find(|p| &p.name == name));

    let phase = if let Some(p) = current_phase {
        p
    } else {
        println!("请先完成开发阶段");
        return Ok(());
    };

    if phase.status != "dev" {
        println!("请先完成开发阶段");
        return Ok(());
    }

    // 渲染 Prompt
    let prompt = render(
        VERIFY_PROMPT,
        &crate::prompts::PromptParams::new()
            .with_file(phase.file.clone()),
    );

    println!("{}", prompt);
    println!();
    println!("验证通过后请执行 !ddd-tool finish-phrase 或 !ddd-tool set-issue");

    Ok(())
}
