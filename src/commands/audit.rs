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

    fn command_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            "使用 Bash工具 执行: {} {name}。审核 @project_docs/specs/ 下所有规格文件的质量和一致性。检查规格是否完整、是否可执行、是否有矛盾。审核通过后等待用户确认阶段计划, 然后调用 `ddd-tool accept` 接受计划。",
            bin
        ))
    }

    fn skill_prompt(&self, bin: &str, name: &str) -> Option<String> {
        Some(format!(
            r#"---
name: "{name}"
description: "审核规格文件的质量和一致性"
---
调用 !`{} {name} 2>&1`
审核 specs/ 下所有规格文件质量和一致性
"#,
            bin
        ))
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
