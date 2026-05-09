use crate::commands::{DddContext, VerifyCmd};
use crate::prompts::render;
use anyhow::Result;

const VERIFY_PROMPT: &str = r#"根据开发计划: @{file} ,并从开发计划中提取对应的规格文档作为资料,然后
1. 对 {name} 的成果代码进行代码审核.
2. 运行所有单元测试
3. 核对spec对代码进行深度事实审核
4. 保证所有功能均已经完整实现, 没有任何占位符实现, 桩实现, 禁止任何的mock
审核完成后输出审核结果. 如果有问题, 先生成修复计划, 并将修复任务按顺序委托给子代理串行执行.
修复完成后重新执行审核任务, 有问题计划并修复, 一直到完全没有问题产生.
也就是按照:
```
while:
  审核
  if 有问题
    修复
  else
    break
```
的逻辑执行, 当全部完成后
提醒是否要执行 /ddd-confirm 确认完成本阶段开发
"#;

pub fn run(_cmd: VerifyCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验状态
    let mut state = ctx.load_state()?;

    let current_name = state.clone().current_phase.unwrap_or("".to_string());

    let phase = state.phrases.iter_mut().find(|p| p.name == current_name);
    let phase = match phase {
        Some(p) => p,
        None => {
            println!("请先完成开发阶段: {}, 停止执行等待用户介入", current_name);
            return Ok(());
        }
    };
    // 更新状态为 verifying
    phase.status = "verifying".to_string();

    // 渲染 Prompt
    let prompt = render(
        VERIFY_PROMPT,
        &crate::prompts::PromptParams::new()
            .with_file(phase.file.clone()).with_name(current_name.clone()),
    );

    println!("{}", prompt);
    // 保存状态
    ctx.save_state(&state.clone())?;

    Ok(())
}
