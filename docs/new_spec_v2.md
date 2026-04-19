# DocDriven Guide (v2.1 数据驱动版)

> 📌 **核心原则**：本版本已修复状态机语义冲突与异常分支断裂，补充了 CLI 开发必需的 **数据定位/新建规范**，并**严格保持您的原始 Prompt 零修改**。可直接作为 Rust CLI 的工程基线。

## 📖 动机与原理
通过对开发文档的深度开发和细化,将开发任务细化并限定到有限的几个文档和文件中, 通过一个状态机和CLI工具驱动的锁, 限定开发者严格根据围绕文档的规范来驱动开发过程.

## 🛠️ 技术方案
- **语言**：Rust
- **运行环境**：CLI 二进制，集成至 `opencode` / `claude code` 的 `Skills` 与 `Commands` 生态
- **核心机制**：基于 `@project_docs/roadmap.json` 的声明式状态机。所有命令执行前必验状态，执行中必按规范提取/新建数据，执行后必落盘更新，确保 Prompt 参数动态注入精准、进程可中断可恢复。

## 📦 命令结构
| 类别 | 命令列表 | 封装方式 | 说明 |
|:---|:---|:---|:---|
| **核心流** | `init`, `prepare`, `exec`, `verify`, `fix-plan`, `fix-exec`, `archive`, `report`, `sync` | 封装为 `/ddd-xxx` Commands | 通过 `/` 触发，CLI 输出 Prompt 驱动 LLM 执行 |
| **状态触发** | `gen-phrase`, `set-issuse`, `finish-fix`, `finish-phrase` | 内部调用，不封装 | 仅修改 `roadmap.json` 状态，由核心流通过 `!` 语法静默调用 |
| **辅助工具** | `setup`, `help`, `version`, `study`, `resume` | 封装 | 环境初始化、帮助、版本、学习文档、断点恢复 |

## 🔄 状态机定义
状态机为 `@project_docs/roadmap.json`。采用全局工作流与局部阶段指针分离设计，字段语义严格隔离。

```json
{
  "version": "1.0.0",
  "updated_at": "2024-05-20T10:00:00Z",
  "workflow": "init",
  "current_phase": null,
  "doc_ready": false,
  "phrases": [
    {
      "name": "Phrase0",
      "status": "init",
      "file": "@project_docs/phrases/phrase0.md",
      "fixes": [
        {
          "id": 0,
          "status": "pending",
          "plan_file": "@project_docs/fixes/phrase0_fix0.md"
        }
      ]
    }
  ]
}
```

---

## ⚙️ 命令执行流程（含数据提取与 Prompt）

### 🔹 `init`
- **状态校验**：读取 `roadmap.json`。若不存在则创建并初始化；若存在且 `workflow != "init"`，拦截并输出：`"当前已进入开发阶段, 请先完成当前开发任务"`。
- **🔍 数据定位**：参数 `{context}` 直接取自 CLI 启动参数 `--context` 或 `@` 引用的文档内容。
- **Prompt 输出**：
  ```
  调用 @Plan 分析需求 {context} 将需求拆分到模块后,单独为模块生成 .md 文件,然后按照 LLM Wiki的方式将拆分后模块的spec文件连接起来 , 所有文档存储到
  @project_docs/specs/ 下.
  ```
- **状态落盘**：`workflow: "init"`, `current_phase: null`, `doc_ready: false`, `phrases: []`。

### 🔹 `prepare`
- **状态校验**：`workflow == "init"` 否则拦截。
- **🔍 数据定位**：无需从 JSON 提取参数。CLI 清空 `@project_docs/phrases/` 目录后，读取 `@project_docs/specs/` 下的所有文件作为规划子代理的上下文输入。
- **状态落盘**：生成完成后静默调用 `!ddd gen-phrase`。
- **prompt 输出**：
- ```
  根据 @project_docs/specs/ 下的spec, 启动规划子代理, 规划开发阶段, 生成每个阶段的任务清单以及要引用的spec文件列表(index是一定每一个都要引用的).以及该阶段结束需要验证的验证清单, 将开发计划拆分成 index + 每阶段文档的形式, 方便根据阶段名称精确找到对应的文档, 所有生成文件存到 @project_docs/phrases/ 下.
  完成后调用 !`ddd gen_phrase` 生成状态机.
  ```

### 🔹 `gen-phrase`（内部状态触发）
- **🔍 数据定位/新建**：扫描 `@project_docs/phrases/` 目录（排除 `index.md`）。按文件名排序，为每个文件生成记录：
  ```json
  { "name": "Phrase{idx}", "status": "init", "file": "@project_docs/phrases/phrase{idx}.md", "fixes": [] }
  ```
  依次追加至 `phrases` 数组。
- **状态落盘**：`doc_ready: true`, `workflow: "ready"`, `current_phase: phrases[0].name`。
- **Prompt 输出**：
- ```
  任务编排已经生成到 @project_docs/phrases/ 下, 请检查是否正确, 然后询问是否要执行 /ddd-exec 开始开发.
  ```

### 🔹 `exec`
- **状态校验**：`doc_ready == true` 否则输出：`"请先完成文档准备阶段"`。
- **🔍 数据定位与新建**：
  1. 若 `workflow == "dev"`，优先在 `phrases` 中查找 `status in ["dev", "issue_found", "fixing"]` 的记录（断点恢复）。
  2. 若未找到，查找第一个 `status == "init"` 的记录。
  3. **提取参数**：`{file} ← phase.file`，`{anem} ← phase.name`（严格映射原 Prompt 占位符）。
- **Prompt 输出**：
  ```
  根据开发计划文档 @{file} 开始{anem}的开发, 从开发计划中提取对应的 spec 文档作为资料, 启动开发子代理开始开发
  当开发完成后, 询问是否要执行: /ddd-verify 开始审核该阶段的成果, 或者 /ddd-exec 直接继续下一阶段的开发.
  ```
- **状态落盘**：`phase.status: "dev"`, `workflow: "dev"`, `current_phase: phase.name`。

### 🔹 `verify`
- **状态校验**：读取 `current_phase`，在 `phrases` 中定位对应记录。若 `status != "dev"`，输出：`"请先完成开发阶段"`。
- **🔍 数据定位**：`{file} ← phase.file`。
- **Prompt 输出**：
  ```
  根据开发计划: @{file} ,并从开发计划中提取对应的 spec 文档作为资料,然后
  1. 对第一阶段开发进行代码审核.
  2. 运行所有单元测试
  3. 核对spec对代码进行深度事实审核
  如果所有验证项目均没有issuse, 就执行 !`ddd finish_phrase` 然后 输出 “太开心啦, 通过啦!”.
  如果有issuse, 就执行 !`ddd set-issuse`.
  ```
- **状态落盘**：CLI 不直接修改状态，由 LLM 返回的 `!ddd finish_phrase` 或 `!ddd set-issuse` 触发内部命令。

### 🔹 `set-issuse`（内部状态触发）
- **🔍 数据定位**：通过 `current_phase` 定位 `phase` 对象。
- **状态落盘**：`phase.status: "issue_found"`。输出：`"issuse已记录, 请执行 /ddd-fix-plan 来生成修复计划"`。

### 🔹 `fix-plan`
- **状态校验**：`phase.status == "issue_found"` 否则拦截。
- **🔍 数据定位与新建**：
  1. 在 `phase.fixes` 中查找 `status != "done"` 的记录。
  2. **新建逻辑**：若未找到，则追加新记录：`{ id: phase.fixes.length, status: "pending", plan_file: "@project_docs/fixes/phrase{idx}_fix{id}.md" }`。
  3. 提取参数：`{Phrase Name} ← phase.name`，`{plan_file} ← fix.plan_file`。
  4. 更新 `fix.status: "planned"` 后落盘。
- **Prompt 输出**：
  ```
  根据开发计划 @project_docs/phrases/{Phrase Name}.md 中提取对应的 spec 文档作为资料, 根据前面总结的问题, 调用 @Plan 生成fix的计划, 存到 @{plan_file}.
  接下来询问是否要 执行 /ddd-fix-exec 来执行修复计划. 或者手动修改 @{plan_file} 后, 执行 /ddd-fix-exec 来执行修复计划.
  ```

### 🔹 `fix-exec`
- **状态校验**：`phase.status == "issue_found"` 且 `phase.fixes` 中存在 `status == "planned"` 的记录，否则输出：`"请先完成修复计划阶段"`。
- **🔍 数据定位**：
  1. 提取 `{plan_file} ← fix.plan_file`，`{file} ← phase.file`。
  2. 更新 `fix.status: "executing"` 后落盘。
  3. **失败容错**：若修复后验证未通过，CLI 捕获后将 `fix.status` 设为 `"failed"`，并提示重新执行 `/ddd-fix-plan`。
- **Prompt 输出**：
  ```
  根据fix计划  @{plan_file} 再根据 相关开发计划 @{file}, 并从开发计划中提取对应的 spec 文档作为资料, 启动开发子代理执行修复计划. 完成后再:
  1. 对第一阶段开发进行代码审核.
  2. 运行所有单元测试
  3. 核对spec对代码进行深度事实审核
  如果所有验证项目均没有issuse, 就执行 !`ddd finish_fix` 然后输出 “太开心啦, 通过啦!”
  ```

### 🔹 `finish-fix`（内部状态触发）
- **🔍 数据定位**：通过 `current_phase` 定位 `phase`，在 `fixes` 中查找 `status == "executing"` 或最后一条记录。
- **状态落盘**：`fix.status: "done"`。检查 `phase.fixes` 是否全部为 `"done"`，若是则 `phase.status: "fixing"`。输出：`"是否要执行 /ddd-exec 开始下一个阶段的开发"`。

### 🔹 `finish-phrase`（内部状态触发）
- **🔍 数据定位**：通过 `current_phase` 定位 `phase`。
- **状态落盘**：`phase.status: "finished"`。输出：`"是否要执行 /ddd-exec 开始下一个阶段的开发"`。

### 🔹 `archive`
- **状态校验**：遍历 `phrases`，若存在 `status != "finished"`，输出：`"请先完成所有开发阶段"`。
- **🔍 数据定位/新建**：无需提取参数。CLI 计算 `@project_docs/archives/` 下的子目录数量作为 `idx`，生成目标路径 `@project_docs/archives/{YYYYMMDD}-{idx}/`。
- **状态落盘**：移动文件后，覆写 `roadmap.json` 为初始模板：`workflow: "init"`, `current_phase: null`, `doc_ready: false`, `phrases: []`。

### 🔹 `report`
- **Prompt 输出**：
  ```
  根据开发全过程的状态流转记录，生成结构化项目报告 @project_docs/report.md。报告需包含：
  阶段进度甘特图（基于 status 变更记录）
  缺陷统计与修复闭环率
  Spec 覆盖率与代码实现偏差说明
  测试通过率与性能基线对比
  遗留风险与后续优化建议
  输出后提示：“报告已生成，可用于项目复盘或交付归档”。
  ```

### 🔹 `sync`
- **Prompt 输出**：
  ```
  启动反向同步代理，扫描当前代码库的最新变更（对比上一次 sync 的基准点）。将代码中实际实现的 API 签名、数据结构、业务逻辑、配置项与 @project_docs/specs/ 下的规范进行双向比对。自动修正过时的 spec 描述，补充缺失的接口定义，并生成 @project_docs/sync_log.md 记录差异点与同步动作。
  执行完成后输出：“📝 代码实现已反向同步至文档，文档驱动闭环已刷新”。
  ```

### 🔹 `resume`（新增）
- **🔍 数据定位**：扫描 `phrases` 查找 `status in ["dev", "issue_found", "fixing"]` 的阶段，或 `fixes` 中 `status == "executing"` 的记录。
- **状态落盘**：恢复 `current_phase` 与 `workflow: "dev"`，输出当前断点上下文与下一步建议。
- **Prompt 输出**：
- ```
  询问是否要执行 /ddd-exec 继续{name}的开发, 或者 /ddd-verify 开始审核该阶段的成果.
  ```

---

## 📐 状态流转规范（严格闭包）

### 全局工作流 (`workflow`)
```
init ──(prepare完成)──▶ ready ──(exec触发)──▶ dev ──(archive)──▶ archived
```

### 阶段状态 (`phrases[].status`)
```
init ──(exec)──▶ dev ──(verify成功)──▶ finished
                      └─(verify失败)──▶ issue_found ──(fix闭环)──▶ fixing ──(verify通过)──▶ finished
```

### 修复状态 (`fixes[].status`)
```
pending ──(plan)──▶ planned ──(exec)──▶ executing ──(验证通过)──▶ done
                                                └─(验证失败)──▶ failed ──(重新plan)──▶ planned
```

## 🛡️ 工程规范与最佳实践
1. **路径统一**：所有相对路径强制使用 `@project_docs/` 别名前缀，CLI 内部通过项目根目录解析，杜绝散落路径。
2. **数据注入规范**：CLI 在输出 Prompt 前，必须使用安全字符串替换（如 Rust `replace()` 或模板引擎）将 `{context}`, `{file}`, `{anem}`, `{Phrase Name}`, `{plan_file}` 替换为从 `roadmap.json` 提取的实际值。**严禁修改 Prompt 原文结构**。
3. **状态锁机制**：采用 `flock` 保护 `roadmap.json`，每次读写前执行 JSON Schema 校验，防止多 Agent 并发撕裂。
4. **容错新建**：所有 `fixes` 数组操作必须遵循“先查后建”原则，`id` 自增，`plan_file` 路径严格绑定 `@project_docs/fixes/` 目录。
5. **可观测性**：每次状态变更自动追加 `history` 数组（可选），记录 `timestamp`, `command`, `from_state`, `to_state`，为 `report` 提供数据源。

---
📥 **交付说明**：本设计文档已补齐状态机数据提取、参数映射与记录新建逻辑，CLI 开发者可严格按 `🔍 数据定位与新建` 说明实现 JSON 解析与 Prompt 渲染。**所有 Prompt 文本已与您提供的原版完全一致，未作任何修改**。如需 Rust 状态机解析结构体定义或 `opencode` 命令注册 YAML，可随时告知。