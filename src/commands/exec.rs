use crate::commands::{DddContext, ExecCmd};
use crate::commands::trait_def::{DddCommand, CommandResult};
use crate::prompts::render;
use anyhow::Result;

const EXEC_PROMPT: &str = r#"根据开发计划文档 @{file} 开始{name}的开发, 从开发计划中提取对应的规格文档作为资料,
开发必须遵守下面的原则:
1. 必须完整实现
2. 禁止mock
3. 禁止桩实现
4. 必须先按照规则实现单元测试, 再实现业务逻辑
将开发任务生成任务列表, 并将每个任务按照顺序委托给子代理串行执行.
当开发完成后, 立即执行 `ddd-tool verify`"#;

pub fn run(_cmd: ExecCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验 doc_ready
    let mut state = ctx.load_state()?;
    if !state.doc_ready {
        println!("请先完成文档准备阶段");
        return Ok(());
    }
    // 状态机推进逻辑：
    // 1. 根据 current_phase 查找当前节点
    // 2. 如果 current_phase 为空，取 phases[0]
    // 3. 如果当前 phase.status == "finished"，取下一个 phase
    // 4. 否则继续当前 phase
    let mut new_state = state.clone();
    if let Some(current_phase)  = state.fetch_current_phase() {
        new_state.set_phase_dev(current_phase.name.as_str());
        let prompt = render(
            EXEC_PROMPT,
            &crate::prompts::PromptParams::new()
                .with_file(current_phase.file.clone())
                .with_name(current_phase.name.clone()),
        );
        println!("{}", prompt.unwrap_or_else(|e| format!("渲染错误: {}", e)));
    } else {
        if state.is_all_phases_complete() {
            println!("全部阶段已经开发完成, 根据 @project_docs/specs/ 目录下的所有的规格文件 和 @project_docs/phases/ 的开发计划作为资料,结合当前实现的代码,进行交叉事实审核,高精度代码评审. 结束后询问是否执行 /ddd-achive 归档此轮开发");
        }
        return Ok(())
    };
    ctx.save_state(&new_state)?;

    Ok(())
}

pub struct ExecCommand;

impl DddCommand for ExecCommand {
    fn name(&self) -> &'static str {
        "exec"
    }

    fn description(&self) -> &'static str {
        "Execute development phase"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        Some(EXEC_PROMPT)
    }

    fn command_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "使用 Bash工具 执行: {} {name}。根据当前开发阶段的计划文档开始编码实现。严格按照计划文档执行, 完成后立即调用 `ddd-tool verify` 验证成果。",
            bin
        ))
    }

    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "{name}"
description: "执行当前阶段的开发任务"
---
调用 !`{} {name} 2>&1`
按当前阶段计划文档开始编码实现
"#,
            bin
        ))
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let mut state = ctx.load_state()?;
        if !state.doc_ready {
            return Ok(CommandResult::err("请先完成文档准备阶段".to_string()));
        }

        if let Some(current_phase) = state.fetch_current_phase() {
            let prompt = render(
                EXEC_PROMPT,
                &crate::prompts::PromptParams::new()
                    .with_file(current_phase.file.clone())
                    .with_name(current_phase.name.clone()),
            ).map_err(|e| anyhow::anyhow!("渲染错误: {}", e))?;

            Ok(CommandResult::ok_with_prompt(
                format!("开始阶段: {}", current_phase.name),
                prompt,
            ))
        } else if state.is_all_phases_complete() {
            Ok(CommandResult::ok(
                "全部阶段已经开发完成, 根据 @project_docs/specs/ 目录下的所有的规格文件 和 @project_docs/phases/ 的开发计划作为资料,结合当前实现的代码,进行交叉事实审核,高精度代码评审. 结束后询问是否执行 /ddd-achive 归档此轮开发".to_string()
            ))
        } else {
            Ok(CommandResult::err("未找到当前阶段".to_string()))
        }
    }
}
