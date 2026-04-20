use crate::commands::{DddContext, FixExecCmd};
use crate::prompts::render;
use anyhow::Result;

const FIX_EXEC_PROMPT: &str = r#"根据fix计划  @{plan_file} 再根据 相关开发计划 @{file}, 并从开发计划中提取对应的 spec 文档作为资料, 启动开发子代理执行修复计划. 完成后再:
1. 对第一阶段开发进行代码审核.
2. 运行所有单元测试
3. 核对spec对代码进行深度事实审核
如果所有验证项目均没有issuse, 就执行 !`ddd-tool finish_fix` 然后输出 "太开心啦, 通过啦!""#;

pub fn run(_cmd: FixExecCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    let state = ctx.load_state()?;

    // 找到 issue_found 的 phase
    let current_phase = state.current_phase.as_ref()
        .and_then(|name| state.phrases.iter().find(|p| &p.name == name));

    let phase = if let Some(p) = current_phase {
        p
    } else {
        println!("请先完成开发阶段");
        return Ok(());
    };

    if phase.status != "issue_found" {
        println!("请先完成开发阶段");
        return Ok(());
    }

    // 查找 planned 的 fix
    let fix = phase.fixes.iter().find(|f| f.status == "planned");

    let (plan_file, file) = if let Some(f) = fix {
        (f.plan_file.clone(), phase.file.clone())
    } else {
        println!("请先完成修复计划阶段");
        return Ok(());
    };

    // 渲染 Prompt
    let prompt = render(
        FIX_EXEC_PROMPT,
        &crate::prompts::PromptParams::new()
            .with_plan_file(plan_file.clone())
            .with_file(file),
    );

    println!("{}", prompt);

    Ok(())
}
