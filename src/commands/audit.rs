use crate::commands::DddContext;
use crate::commands::trait_def::{DddCommand, CommandResult};
use anyhow::Result;

const AUDIT_PROMPT: &str = r#"根据 @project_docs/specs/ 目录下的所有的规格文件, 和 @project_docs/phases/ 的开发计划作为资料,
结合当前实现的代码, 进行交叉事实审核, 高精度代码评审.
审核要点:
1. 规格文档与实际代码的一致性
2. 开发计划是否完整覆盖所有规格
3. 代码实现是否符合规格要求
4. 是否有遗漏的功能点
5. 是否有违反设计原则的实现
将评审的任务委托给子代理执行.
当评审完成后, 如果有问题, 就按照优先级,委托给子代理串行执行修复.
全部完成后立即执行 `ddd-tool accept` 批准设计
"#;

pub fn run(_cmd: crate::commands::AuditCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验 specs 目录存在
    let specs_dir = ctx.project_root.join("project_docs").join("specs");
    if !specs_dir.exists() {
        println!("规格文档目录不存在: {}", specs_dir.display());
        return Ok(());
    }

    // 渲染 Prompt
    //let prompt = render(specs_dir.to_string_lossy().as_ref());
    println!("{}", AUDIT_PROMPT);

    Ok(())
}

pub struct AuditCommand;

impl DddCommand for AuditCommand {
    fn name(&self) -> &'static str {
        "audit"
    }

    fn description(&self) -> &'static str {
        "Audit specs and plans"
    }

    fn prompt_template(&self) -> Option<&'static str> {
        Some(AUDIT_PROMPT)
    }

    fn execute(&self, ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        let specs_dir = ctx.project_root.join("project_docs").join("specs");
        if !specs_dir.exists() {
            return Ok(CommandResult::err(format!(
                "规格文档目录不存在: {}",
                specs_dir.display()
            )));
        }

        Ok(CommandResult::ok_with_prompt(
            "审计 prompt 已生成".to_string(),
            AUDIT_PROMPT.to_string(),
        ))
    }
}

#[allow(dead_code)]
fn render(specs_dir: &str) -> String {
    format!(
        r#"根据 {} 目录下的所有的规格文件, 和 @project_docs/phases/ 的开发计划作为资料,
结合当前实现的代码, 进行交叉事实审核, 高精度代码评审.
审核要点:
1. 规格文档与实际代码的一致性
2. 开发计划是否完整覆盖所有规格
3. 代码实现是否符合规格要求
4. 是否有遗漏的功能点"#, specs_dir
    )
}
