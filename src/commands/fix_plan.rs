use crate::commands::{DddContext, FixPlanCmd};
use crate::prompts::render;
use anyhow::Result;

const FIX_PLAN_PROMPT: &str = r#"根据开发计划 @project_docs/phrases/{Phrase Name}.md 中提取对应的 spec 文档作为资料, 根据前面总结的问题, 调用 @Plan 生成fix的计划, 存到 @{plan_file}.
接下来询问是否要 执行 /ddd-fix-exec 来执行修复计划. 或者手动修改 @{plan_file} 后, 执行 /ddd-fix-exec 来执行修复计划."#;

pub fn run(_cmd: FixPlanCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    let state = ctx.load_state()?;

    // 找到 issue_found 的 phase
    let current_phase = state.current_phase.as_ref()
        .and_then(|name| state.phrases.iter().find(|p| &p.name == name));

    let phase = if let Some(p) = current_phase {
        p
    } else {
        println!("请先完成开发阶段");
        return Ok(());
    };

    if phase.status != "issue_found" {
        println!("请先完成开发阶段");
        return Ok(());
    }

    // 查找或创建 fix
    let fix = phase.fixes.iter().find(|f| f.status != "done");

    let (_, plan_file) = if let Some(f) = fix {
        (f.id, f.plan_file.clone())
    } else {
        // 新建 fix
        let new_id = phase.fixes.len() as u32;
        let new_plan_file = format!(
            "@project_docs/fixes/{}_fix{}.md",
            phase.name.replace("Phrase", "phrase"),
            new_id
        );
        (new_id, new_plan_file)
    };

    // 渲染 Prompt
    let prompt = render(
        FIX_PLAN_PROMPT,
        &crate::prompts::PromptParams::new()
            .with_phrase_name(phase.name.clone())
            .with_plan_file(plan_file.clone()),
    );

    println!("{}", prompt);
    println!();
    println!("修复计划文件: {}", plan_file);

    Ok(())
}
