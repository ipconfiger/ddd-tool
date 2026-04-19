# DocDriven Engineering Specification

## 工程规范与最佳实践

### 1. 路径统一

所有相对路径强制使用 `@project_docs/` 别名前缀，CLI 内部通过项目根目录解析，杜绝散落路径。

### 2. 数据注入规范

CLI 在输出 Prompt 前，必须使用安全字符串替换（如 Rust `replace()` 或模板引擎）将 `{context}`, `{file}`, `{anem}`, `{Phrase Name}`, `{plan_file}` 替换为从 `roadmap.json` 提取的实际值。**严禁修改 Prompt 原文结构**。

### 3. 状态锁机制

采用 `flock` 保护 `roadmap.json`，每次读写前执行 JSON Schema 校验，防止多 Agent 并发撕裂。

### 4. 容错新建

所有 `fixes` 数组操作必须遵循"先查后建"原则，`id` 自增，`plan_file` 路径严格绑定 `@project_docs/fixes/` 目录。

### 5. 可观测性

每次状态变更自动追加 `history` 数组（可选），记录 `timestamp`, `command`, `from_state`, `to_state`，为 `report` 提供数据源。

## 目录结构

```
@project_docs/
├── roadmap.json          # 状态机定义
├── specs/                # 需求规格文档
│   ├── index.md
│   ├── spec_core.md
│   ├── spec_commands.md
│   ├── spec_data_flow.md
│   ├── spec_state_machine.md
│   └── spec_engineering.md
├── phrases/              # 开发阶段文档
│   └── phrase{N}.md
├── fixes/                # 修复计划文档
│   └── phrase{N}_fix{M}.md
├── archives/             # 归档目录
│   └── {YYYYMMDD}-{idx}/
├── report.md             # 项目报告
└── sync_log.md           # 同步日志
```

## 相关文档

- 核心机制：[@project_docs/specs/spec_core.md](spec_core.md)
- 命令定义：[@project_docs/specs/spec_commands.md](spec_commands.md)
- 数据流动：[@project_docs/specs/spec_data_flow.md](spec_data_flow.md)
- 状态流转：[@project_docs/specs/spec_state_machine.md](spec_state_machine.md)
