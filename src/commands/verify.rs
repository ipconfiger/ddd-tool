use crate::commands::{DddContext, VerifyCmd};
use crate::commands::trait_def::{DddCommand, CommandResult};
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
的逻辑执行, 当等待全部完成后,立即调用 `ddd-tool confirm`"#;

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

    let phase = state.phases.iter_mut().find(|p| p.name == current_name);
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

    println!("{}", prompt.unwrap_or_else(|e| format!("渲染错误: {}", e)));
    // 保存状态
    ctx.save_state(&state.clone())?;

    Ok(())
}

pub struct VerifyCommand;

impl DddCommand for VerifyCommand {
    fn name(&self) -> &'static str {
        "verify"
    }

    fn description(&self) -> &'static str {
        "Verify phase成果"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        Some(VERIFY_PROMPT)
    }

    fn command_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "使用 Bash工具 执行: {} {name}。验证当前阶段开发成果是否符合规格要求。检查代码质量、测试覆盖、文档完整性。根据验证结果决定是否通过, 通过后立即调用 `ddd-tool confirm` 推进到下一阶段。",
            bin
        ))
    }

    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "{name}"
description: "验证当前阶段开发成果是否符合规格要求"
---
调用 !`{} {name} 2>&1`
验证当前阶段代码质量和规格符合度
"#,
            bin
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let mut state = ctx.load_state()?;

        let current_name = state.clone().current_phase.unwrap_or("".to_string());

        let phase = state.phases.iter_mut().find(|p| p.name == current_name);
        let phase = match phase {
            Some(p) => p,
            None => {
                return Ok(CommandResult::err(format!(
                    "请先完成开发阶段: {}, 停止执行等待用户介入",
                    current_name
                )))
            }
        };

        phase.status = "verifying".to_string();

        let prompt = render(
            VERIFY_PROMPT,
            &crate::prompts::PromptParams::new()
                .with_file(phase.file.clone())
                .with_name(current_name.clone()),
        ).map_err(|e| anyhow::anyhow!("渲染错误: {}", e))?;

        ctx.save_state(&state)?;

        Ok(CommandResult::ok_with_prompt(
            format!("验证阶段: {}", current_name),
            prompt,
        ))
    }
}
