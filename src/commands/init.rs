use crate::commands::{DddContext, InitCmd};
use crate::prompts::render;
use anyhow::Result;
use std::fs;

const INIT_PROMPT: &str = r#"分析需求:{context}, 按照规格设计原则:
按照MVP设计原则与KISS原则, 实现最小化可执行的原则, 如无必要不要增加实体.
### 必须
1. **分层定界**：总章明确定义 In/Out-of-Scope 与量化成功指标；子模块独立描述自身职责。
2. **闭环流转**：每个模块必须包含前置条件、正常流、异常流（含超时/重试/降级/幂等）。
3. **量化一切**：拒绝“高可用/快”，全部转为绝对值（如 QPS、P95延迟、RPO/RTO）。
4. **端到端追溯**：建立 `需求ID ↔ 接口 ↔ 数据表 ↔ 测试用例` 的唯一映射矩阵。
5. **完备数据契约**：定义字段约束、索引策略、生命周期（软删除/归档）及全局唯一错误码。
6. **可执行验收**：使用 Given-When-Then 格式定义每条规则的通过条件。
7. **高内聚低耦合**: 模块设计必须满足高内聚低耦合的原则
### 建议
1. **图表代文**：架构用 C4/数据流，交互用时序图，状态用状态机，杜绝复杂逻辑纯文字描述。
2. **标准化动词**：通篇使用 RFC 2119 词汇（MUST / SHOULD / MAY）替代“建议/尽量”。
3. **Docs as Code**：使用 Markdown + Git 管理，配置 PR Review 与自动化 Lint 检查。
4. **闭环评审**：产研测三方对照文档评审，纪要及遗留问题（需有Owner与Deadline）归档入库。
### 禁止
1. **禁止 YAGNI**：不写当前迭代不用的未来功能，不隐式扩大范围。
2. **禁止规定实现**：不写具体类名、设计模式或底层算法（架构图只画模块边界与依赖）。
3. **禁止主观形容词**：不出现任何无法直接写成断言的词汇。
4. **禁止未闭环项**：文档中不允许存在无结论的 TBD 或待讨论。
5. **禁止割裂交付**：不允许代码已上线但数据模型/接口规范与文档脱节。
---
*附：极简文件结构模板（满足上述规则的最小集）*
*   `project_docs/specs/SPEC_INDEX.md`：痛点指标 / 范围边界 / 架构图 / 依赖与版本
*   `project_docs/specs/xxx.md`：用例流转(时序/状态机) / 接口契约 / 错误码 / 数据模型 / AC用例与追溯矩阵
---
将规格设计的任务,委托的子代理独立生成.
在每个模块的规格文件头部创建到SPEC_INDEX.md的双向 wiki-link 链接,将文档连接起来.
所有文档存储到 @project_docs/specs/ 目录下. 完成后提醒调用 /ddd-prepare 生成开发计划"#;

pub fn run(cmd: InitCmd) {
    if let Err(e) = do_run(cmd) {
        eprintln!("错误: {}", e);
    }
}

fn do_run(cmd: InitCmd) -> Result<()> {
    let ctx = DddContext::new()?;

    // 校验状态
    let state = ctx.load_state()?;
    if state.workflow != "init" {
        println!("当前已进入开发阶段, 请先完成当前开发任务");
        return Ok(());
    }

    // 读取 context 文档内容
    let context_value = if let Some(ref context_path) = cmd.context {
        let resolved = ctx.resolve_path(context_path);
        if resolved.exists() {
            fs::read_to_string(&resolved).unwrap_or_else(|_| context_path.clone())
        } else {
            context_path.clone()
        }
    } else {
        "未提供需求文档".to_string()
    };

    // 渲染 Prompt
    let prompt = render(
        INIT_PROMPT,
        &crate::prompts::PromptParams::new().with_context(context_value),
    );

    println!("{}", prompt);

    // 保存状态
    ctx.save_state(&state)?;

    Ok(())
}
