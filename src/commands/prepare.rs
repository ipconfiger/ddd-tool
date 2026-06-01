use crate::commands::{DddContext, PrepareCmd};
use crate::prompts::render;
use anyhow::Result;
use std::fs;

const PREPARE_PROMPT: &str = r#"根据 @project_docs/specs/ 下的spec, 按照开发计划的需求:
### 必须
1. **ID全链路绑定**：需求(REQ)、任务(TASK)、验收(AC)必须使用唯一标识贯穿，严禁脱离追溯。
2. **结构化拆解**：WBS（任务分解）必须使用列表或表格，每个任务节点必须指向具体的实施动作。
3. **显式划界**：必须明确当前计划的 In-Scope、Out-of-Scope 及前置假设条件。
4. **闭环验证**：每个任务/模块必须附带具体的验证步骤与可直接勾选的验收清单。
5. **高内聚低耦合**: 模块设计必须满足高内聚低耦合的原则

### 建议
1. **元数据前置**：在计划顶部集中声明阶段目标、架构约束及技术栈选型。
2. **机读化表达**：配置、依赖、接口等结构化数据，优先使用 YAML/JSON 代码块或 Markdown 表格呈现。
3. **指令数据隔离**：若用于喂给AI，必须将“生成规则（Prompt）”与“业务输入数据”物理分区块存放。

### 禁止
1. **禁止模糊动词**：任务描述中不允许出现“优化”“完善”“处理”等无法直接判定完成状态的词汇。
2. **禁止上下文缺失**：不允许在未声明架构与技术栈的情况下，直接输出孤立的任务列表。

委托任务到子代理, 规划开发阶段, 串行按照顺序生成每个阶段的开发计划文件, 其中必须包含
1  任务清单(包含详细的执行步骤)以及要在头部列表引用的规格文件(index是一定每一个都要引用的).
2. 该阶段结束需要验证的验证清单,
将开发计划按照 {idx}_{name}.md 的命名规则, 存到 @project_docs/phases/ 下.
**important** idx 从1开始.
完成后提示调用 /ddd-accept 生成状态机."#;

pub fn run(_cmd: PrepareCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验状态
    let state = ctx.load_state()?;
    if state.workflow != "init" {
        println!("当前已进入开发阶段, 请先完成当前开发任务");
        return Ok(());
    }

    // 清空 phases 目录
    let phases_dir = ctx.project_root.join("project_docs").join("phases");
    if phases_dir.exists() {
        fs::remove_dir_all(&phases_dir)?;
    }
    fs::create_dir_all(&phases_dir)?;

    // 渲染 Prompt
    let prompt = render(
        PREPARE_PROMPT,
        &crate::prompts::PromptParams::new(),
    );

    println!("{}", prompt);

    // 保存状态
    ctx.save_state(&state)?;

    Ok(())
}
