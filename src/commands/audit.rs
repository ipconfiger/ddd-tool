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
"#;

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

    fn command_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "加载 Skill ddd-{name}, 执行技能开始评审, 执行完成后, 调用 Skill ddd-accept",
        ))
    }

    fn skill_prompt(&self, _bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "ddd-{name}"
description: "审核规格文件的质量和一致性"
---
{}
"#,
            AUDIT_PROMPT
        ))
    }

    fn execute(&self, _ctx: &DddContext, _args: &str) -> Result<CommandResult> {
        // let specs_dir = ctx.project_root.join("project_docs").join("specs");
        // if !specs_dir.exists() {
        //     return Ok(CommandResult::err(format!(
        //         "规格文档目录不存在: {}",
        //         specs_dir.display()
        //     )));
        // }
        //
        // Ok(CommandResult::ok_with_prompt(
        //     "审计 prompt 已生成".to_string(),
        //     AUDIT_PROMPT.to_string(),
        // ))
        Ok(CommandResult::ok(""))
    }
}
