use crate::commands::DddContext;
use anyhow::Result;

pub fn run(_cmd: crate::commands::ConfirmCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    let mut state = ctx.load_state()?;

    if !state.doc_ready {
        println!("ECHO:请先完成文档准备阶段: 调用 /ddd-accept` 批准开发计划. 停止执行!");
        return Ok(());
    }

    if state.current_phase.is_none() {
        println!("尚未启动开发");
        return Ok(());
    }

    match state.advance_phase()? {
        Some(next) => {
            let next_name = next.name.clone();
            ctx.save_state(&state)?;
            println!("接下来调用 /ddd-exec 开始实现 {}", next_name);
        }
        None => {
            if state.is_all_phases_complete() {
                println!("全部阶段已经开发完成, 根据 @project_docs/specs/ 目录下的所有的规格文件 和 @project_docs/phases/ 的开发计划作为资料,结合当前实现的代码,进行交叉事实审核,高精度代码评审. 结束后询问是否执行 /ddd-achive 归档此轮开发");
            }
        }
    }
    Ok(())
}
