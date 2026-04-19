# DocDriven Data Flow Specification

## 数据定位规范

### 通用数据定位模式

| 命令 | 数据源 | 提取参数 |
|------|-------|---------|
| `init` | CLI 启动参数 `--context` 或 `@` 引用文档 | `{context}` |
| `exec` | `phase.file` | `{file}`, `{anem}` |
| `verify` | `phase.file` | `{file}` |
| `fix-plan` | `phase.name`, `fix.plan_file` | `{Phrase Name}`, `{plan_file}` |
| `fix-exec` | `fix.plan_file`, `phase.file` | `{plan_file}`, `{file}` |
| `resume` | `phase.name` | `{name}` |

## Prompt 参数映射

### 占位符定义

| 占位符 | 来源 | 说明 |
|--------|------|------|
| `{context}` | CLI 启动参数 | 原始需求上下文 |
| `{file}` | `phase.file` | 当前阶段文档路径 |
| `{anem}` | `phase.name` | 当前阶段名称（原文占位符） |
| `{Phrase Name}` | `phase.name` | 当前阶段名称 |
| `{plan_file}` | `fix.plan_file` | 修复计划文件路径 |
| `{name}` | `phase.name` | 当前阶段名称（resume 用） |

### 安全替换规范

CLI 在输出 Prompt 前，必须使用安全字符串替换（如 Rust `replace()` 或模板引擎）将占位符替换为实际值。**严禁修改 Prompt 原文结构**。

## roadmap.json Schema

```json
{
  "version": "string",
  "updated_at": "string (ISO8601)",
  "workflow": "string (init|ready|dev|archived)",
  "current_phase": "string | null",
  "doc_ready": "boolean",
  "phrases": [
    {
      "name": "string",
      "status": "string (init|dev|issue_found|fixing|finished)",
      "file": "string (path)",
      "fixes": [
        {
          "id": "number",
          "status": "string (pending|planned|executing|done|failed)",
          "plan_file": "string (path)"
        }
      ]
    }
  ]
}
```

## 内部状态触发命令

### gen-phrase（内部状态触发）

- **数据定位/新建**：扫描 `@project_docs/phrases/` 目录（排除 `index.md`）。按文件名排序，为每个文件生成记录：
  ```json
  { "name": "Phrase{idx}", "status": "init", "file": "@project_docs/phrases/phrase{idx}.md", "fixes": [] }
  ```
  依次追加至 `phrases` 数组。
- **状态落盘**：`doc_ready: true`, `workflow: "ready"`, `current_phase: phrases[0].name`。

### set-issuse（内部状态触发）

- **数据定位**：通过 `current_phase` 定位 `phase` 对象。
- **状态落盘**：`phase.status: "issue_found"`。输出：`"issuse已记录, 请执行 /ddd-fix-plan 来生成修复计划"`。

### finish-fix（内部状态触发）

- **数据定位**：通过 `current_phase` 定位 `phase`，在 `fixes` 中查找 `status == "executing"` 或最后一条记录。
- **状态落盘**：`fix.status: "done"`。检查 `phase.fixes` 是否全部为 `"done"`，若是则 `phase.status: "fixing"`。输出：`"是否要执行 /ddd-exec 开始下一个阶段的开发"`。

### finish-phrase（内部状态触发）

- **数据定位**：通过 `current_phase` 定位 `phase`。
- **状态落盘**：`phase.status: "finished"`。输出：`"是否要执行 /ddd-exec 开始下一个阶段的开发"`。

## 相关文档

- 核心机制：[@project_docs/specs/spec_core.md](spec_core.md)
- 命令定义：[@project_docs/specs/spec_commands.md](spec_commands.md)
- 状态流转：[@project_docs/specs/spec_state_machine.md](spec_state_machine.md)
- 工程规范：[@project_docs/specs/spec_engineering.md](spec_engineering.md)
