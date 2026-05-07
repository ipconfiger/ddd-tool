use crate::commands::{DddContext, VerifyCmd};
use crate::prompts::render;
use anyhow::Result;

const VERIFY_PROMPT: &str = r#"根据开发计划: @{file} ,并从开发计划中提取对应的 spec 文档作为资料,然后
1. 对第一阶段开发进行代码审核.
2. 运行所有单元测试
3. 核对spec对代码进行深度事实审核
审核完成后输出审核结果."#;

pub fn run(_cmd: VerifyCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验状态
    let mut state = ctx.load_state()?;

    let current_name = match state.current_phase.as_ref() {
        Some(n) => n.clone(),
        None => {
            println!("请先完成开发阶段");
            return Ok(());
        }
    };

    let phase = state.phrases.iter_mut().find(|p| p.name == current_name);
    let phase = match phase {
        Some(p) => p,
        None => {
            println!("请先完成开发阶段");
            return Ok(());
        }
    };

    if phase.status != "dev" {
        println!("请先完成开发阶段");
        return Ok(());
    }

    // 更新状态为 verifying
    phase.status = "verifying".to_string();

    // 渲染 Prompt
    let prompt = render(
        VERIFY_PROMPT,
        &crate::prompts::PromptParams::new()
            .with_file(phase.file.clone()),
    );

    println!("{}", prompt);

    // 保存状态
    ctx.save_state(&state)?;

    Ok(())
}
