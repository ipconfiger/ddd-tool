use crate::commands::DddContext;
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::Result;

const VERIFY_PROMPT: &str = r#"根据本阶段开发计划文档, 并从开发计划中提取对应的规格文档作为资料,然后
1. 对当前阶段的成果代码进行代码审核.
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
的逻辑执行"#;

pub struct VerifyCommand;

impl DddCommand for VerifyCommand {
    fn name(&self) -> &'static str {
        "ddd-verify"
    }

    fn description(&self) -> &'static str {
        "Verify phase成果"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        Some(VERIFY_PROMPT)
    }

    fn command_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "加载Skill {name}, 执行技能"
        ))
    }

    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "{name}"
description: "验证当前阶段开发成果是否符合规格要求"
---
调用 Base !`{} {name} 2>&1` 从返回里获得当前阶段名称,
如果返回的阶段名称:
  根据:{} 验证当前阶段代码质量和规格符合度
  如果执行完毕通过审核, 就 加载 Skill ddd-confirm 并执行;
如果返回"请先完成开发阶段..."就停止,等待用户介入,并提示: 请先完成开发阶段"阶段名称"
"#,
            bin,
            VERIFY_PROMPT
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

        ctx.save_state(&state)?;

        Ok(CommandResult::ok(
            format!("{}", current_name),
        ))
    }
}
