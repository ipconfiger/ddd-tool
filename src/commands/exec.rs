use crate::commands::{DddContext, ExecCmd};
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
        println!("{}", prompt);
    } else {
        if state.is_all_phases_complete() {
            println!("全部阶段已经开发完成, 根据 @project_docs/specs/ 目录下的所有的规格文件 和 @project_docs/phases/ 的开发计划作为资料,结合当前实现的代码,进行交叉事实审核,高精度代码评审. 结束后询问是否执行 /ddd-achive 归档此轮开发");
        }
        return Ok(())
    };
    ctx.save_state(&new_state)?;

    Ok(())
}
