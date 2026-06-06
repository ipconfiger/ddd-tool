use crate::commands::{DddContext, FinalCmd};
use crate::prompts::render;
use anyhow::Result;

const VERIFY_PROMPT: &str = r#"根据全部开发计划: @project_docs/phases/ 以及全部开发规格 @project_docs/specs/ 对当前已经实现的代码进行
1. 进行代码审核.
2. 运行所有单元测试
3. 核对开发规格对代码进行深度事实审核
4. 保证所有功能均已经完整实现, 没有任何占位符实现, 桩实现, 禁止任何的mock
5. 冷启动冒烟测试
6. 针对启动入口的初始化顺序显式审核
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
提醒是否要执行 /ddd-confirm 确认本阶段开发, 进入下一个阶段
"#;

pub fn run(_cmd: FinalCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    // 校验状态
    let state = ctx.load_state()?;
    if !state.is_all_phases_complete() {
        println!("请先完成所有开发阶段, 停止执行!");
    }
    // 渲染 Prompt
    let prompt = render(
        VERIFY_PROMPT,
        &crate::prompts::PromptParams::new()
            .with_name("all".to_string()),
    );
    println!("{}", prompt.unwrap_or_else(|e| format!("渲染错误: {}", e)));
    // 保存状态
    //ctx.save_state(&state)?;

    Ok(())
}
