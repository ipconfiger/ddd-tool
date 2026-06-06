use crate::commands::DddContext;
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::Result;

const EXEC_PROMPT: &str = r#"
### 执行必须遵循的原则
从开发计划中提取对应的规格文档作为资料,
开发必须遵守下面的原则:
1. 必须完整实现
2. 禁止mock
3. 禁止桩实现
4. 必须先按照规则实现单元测试, 再实现业务逻辑
将开发任务生成任务列表, 并将每个任务按照依赖的关系委托给子代理执行."#;

pub struct ExecCommand;

impl DddCommand for ExecCommand {
    fn name(&self) -> &'static str {
        "ddd-exec"
    }

    fn description(&self) -> &'static str {
        "Execute development phase"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        Some(EXEC_PROMPT)
    }

    fn command_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "加载Skill {name}, 执行技能",
        ))
    }

    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "{name}"
description: "执行当前阶段的开发任务"
---
调用 Bash !`{} {name} 2>&1` 获取当前执行阶段名称, 如果返回阶段名称,就:
  {}
  按当前阶段计划文档开始编码实现, 完成后调用 Skill ddd-verify;
如果返回的"已经全部完成" 就调用 Skill ddd-final;
如果返回"请先完成文档准备阶段",就停止执行,提示用户: 请先完成文档准备阶段
"#,
            bin,
            EXEC_PROMPT
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let mut state = ctx.load_state()?;
        if !state.doc_ready {
            return Ok(CommandResult::err("请先完成文档准备阶段".to_string()));
        }

        if let Some(current_phase) = state.fetch_current_phase() {
            let _ = current_phase.status == "dev";
            let phase_name = current_phase.name.to_string();
            ctx.save_state(&state)?;
            Ok(CommandResult::ok(phase_name))
        } else if state.is_all_phases_complete() {
            Ok(CommandResult::ok(
                "已经全部完成".to_string()
            ))
        } else {
            Ok(CommandResult::err("未找到当前阶段".to_string()))
        }
    }
}
