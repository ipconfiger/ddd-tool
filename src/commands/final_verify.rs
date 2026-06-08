use crate::commands::DddContext;
use crate::commands::trait_def::{DddCommand, CommandResult};
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
的逻辑执行, 对所有已完成阶段进行最终交叉验证。检查各阶段之间的集成一致性、整体规格覆盖率。完成后输出最终验证报告。
"#;

pub struct FinalVerifyCommand;

impl DddCommand for FinalVerifyCommand {
    fn name(&self) -> &'static str {
        "final"
    }

    fn description(&self) -> &'static str {
        "Finalize verify for all phases"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        Some(VERIFY_PROMPT)
    }

    fn command_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "加载 Skill ddd-{name}, 执行技能",
        ))
    }

    fn skill_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "ddd-{name}"
description: "对所有阶段进行最终交叉验证"
---
{},
完成后询问是否要执行 /ddd-archive 归档本次开发任务
"#,
            VERIFY_PROMPT
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        // 校验状态
        let state = ctx.load_state()?;
        if !state.is_all_phases_complete() {
            return Ok(CommandResult::err("请先完成所有开发阶段, 停止执行!"));
        }
        // 渲染 Prompt
        let prompt = render(
            VERIFY_PROMPT,
            &crate::prompts::PromptParams::new()
                .with_name("all".to_string()),
        );
        let rendered = prompt.unwrap_or_else(|e| format!("渲染错误: {}", e));
        // 保存状态 (KEEP THIS COMMENTED OUT as requested)
        //ctx.save_state(&state)?;
        Ok(CommandResult::ok_with_prompt(rendered.clone(), rendered))
    }
}
