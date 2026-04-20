use crate::commands::{DddContext, ExecCmd};
use crate::prompts::render;
use anyhow::Result;

const EXEC_PROMPT: &str = r#"根据开发计划文档 @{file} 开始{anem}的开发, 从开发计划中提取对应的 spec 文档作为资料, 启动开发子代理开始开发
当开发完成后, 询问是否要执行: /ddd-verify 开始审核该阶段的成果, 或者 /ddd-exec 直接继续下一阶段的开发."#;

pub fn run(_cmd: ExecCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验 doc_ready
    let state = ctx.load_state()?;
    if !state.doc_ready {
        println!("请先完成文档准备阶段");
        return Ok(());
    }

    // 状态机推进逻辑：
    // 1. 根据 current_phase 查找当前节点
    // 2. 如果 current_phase 为空，取 phases[0]
    // 3. 如果当前 phase.status == "finished"，取下一个 phase
    // 4. 否则继续当前 phase

    let current_phase = if let Some(name) = &state.current_phase {
        state.phases.iter().find(|p| &p.name == name)
    } else {
        state.phases.first()
    };

    let (file, name) = match current_phase {
        Some(phase) if phase.status == "finished" => {
            // 找下一个 phase
            let idx = state.phases.iter().position(|p| p.name == phase.name);
            let next = idx.and_then(|i| state.phases.get(i + 1));
            match next {
                Some(p) => (p.file.clone(), p.name.clone()),
                None => {
                    println!("所有阶段已完成，无需继续开发");
                    return Ok(());
                }
            }
        }
        Some(phase) => (phase.file.clone(), phase.name.clone()),
        None => {
            println!("阶段列表为空");
            return Ok(());
        }
    };

    // 更新状态
    let mut new_state = state.clone();
    new_state.workflow = "dev".to_string();
    new_state.current_phase = Some(name.clone());

    // 找到并更新 phase status
    if let Some(phase) = new_state.phases.iter_mut().find(|p| p.name == name) {
        if phase.status == "init" {
            phase.status = "dev".to_string();
        }
    }

    ctx.save_state(&new_state)?;

    // 渲染 Prompt
    let prompt = render(
        EXEC_PROMPT,
        &crate::prompts::PromptParams::new()
            .with_file(file)
            .with_anem(name),
    );

    println!("{}", prompt);

    Ok(())
}
