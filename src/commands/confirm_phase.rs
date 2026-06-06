use crate::commands::DddContext;
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::Result;

pub struct ConfirmCommand;

impl DddCommand for ConfirmCommand {
    fn name(&self) -> &'static str {
        "confirm"
    }

    fn description(&self) -> &'static str {
        "Confirm phase completion, advance to next"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        None
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let mut state = ctx.load_state()?;

        if !state.doc_ready {
            return Ok(CommandResult::ok("ECHO:请先完成文档准备阶段: 调用 /ddd-accept` 批准开发计划. 停止执行, 等待用户确认!"));
        }

        if state.current_phase.is_none() {
            return Ok(CommandResult::ok("尚未启动开发"));
        }
        if state.is_current_init() {
            return Ok(CommandResult::ok("等待用户输入"));
        }

        let this_name = state.current_phase.as_ref().unwrap().to_string();

        match state.advance_phase()? {
            Some(next) => {
                let next_name = next.name.clone();
                state.current_phase = Some(next_name.to_string());
                ctx.save_state(&state)?;
                Ok(CommandResult::ok(format!("开始实现 {}, 立即调用 `ddd-tool exec`", next_name)))
            }
            None => {
                state.set_phase_finished(this_name.as_str());
                ctx.save_state(&state)?;
                if state.is_all_phases_complete() {
                    Ok(CommandResult::ok("全部阶段已经开发完成, 根据 @project_docs/specs/ 目录下的所有的规格文件 和 @project_docs/phases/ 的开发计划作为资料,结合当前实现的代码,进行交叉事实审核,高精度代码评审. 结束后询问是否执行 /ddd-achive 归档此轮开发"))
                } else {
                    Ok(CommandResult::ok(format!("阶段 {} 已完成", this_name)))
                }
            }
        }
    }
}

pub fn run(_cmd: crate::commands::ConfirmCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    let mut state = ctx.load_state()?;

    if !state.doc_ready {
        println!("ECHO:请先完成文档准备阶段: 调用 /ddd-accept` 批准开发计划. 停止执行, 等待用户确认!");
        return Ok(());
    }

    if state.current_phase.is_none() {
        println!("尚未启动开发");
        return Ok(());
    }
    if state.is_current_init() {
        println!("等待用户输入");
        return Ok(());
    }

    let this_name = state.current_phase.as_ref().unwrap().to_string();

    match state.advance_phase()? {
        Some(next) => {
            let next_name = next.name.clone();
            state.current_phase = Some(next_name.to_string());
            ctx.save_state(&state)?;
            println!("开始实现 {}, 立即调用 `ddd-tool exec`", next_name);
        }
        None => {
            state.set_phase_finished(this_name.as_str());
            ctx.save_state(&state)?;
            if state.is_all_phases_complete() {
                println!("全部阶段已经开发完成, 根据 @project_docs/specs/ 目录下的所有的规格文件 和 @project_docs/phases/ 的开发计划作为资料,结合当前实现的代码,进行交叉事实审核,高精度代码评审. 结束后询问是否执行 /ddd-achive 归档此轮开发");
            }
        }
    }
    Ok(())
}
