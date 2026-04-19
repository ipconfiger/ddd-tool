# DocDriven Guide - LLM Wiki Index

> 文档驱动开发框架 (v2.1 数据驱动版)

## 项目概述

DocDriven 是一个基于声明式状态机的文档驱动开发框架，通过 CLI 工具锁定的机制，将开发者的工作限定在围绕文档的规范中。

**核心技术**：
- 语言：Rust
- 运行环境：CLI 二进制，集成至 `opencode` / `claude code` 的 `Skills` 与 `Commands` 生态

## 模块索引

| 模块 | 文件 | 说明 |
|------|------|------|
| **核心机制** | [spec_core.md](spec_core.md) | 状态机定义、workflow/phases/status 语义 |
| **命令定义** | [spec_commands.md](spec_commands.md) | 所有命令的详细定义、执行流程、状态校验规则 |
| **数据流动** | [spec_data_flow.md](spec_data_flow.md) | 数据定位规范、Prompt 参数映射、JSON Schema |
| **状态流转** | [spec_state_machine.md](spec_state_machine.md) | 状态流转图、workflow/phases/fixes 状态机定义 |
| **工程规范** | [spec_engineering.md](spec_engineering.md) | 工程规范、最佳实践、安全约束 |

## 命令索引

### 核心流命令

| 命令 | 说明 |
|------|------|
| `/ddd-init` | 初始化项目，读取需求文档 |
| `/ddd-prepare` | 生成开发计划 |
| `/ddd-exec` | 执行开发 |
| `/ddd-verify` | 验证阶段成果 |
| `/ddd-fix-plan` | 生成修复计划 |
| `/ddd-fix-exec` | 执行修复 |
| `/ddd-archive` | 归档项目 |
| `/ddd-report` | 生成项目报告 |
| `/ddd-sync` | 同步代码与文档 |

### 辅助命令

| 命令 | 说明 |
|------|------|
| `/ddd-resume` | 断点恢复 |

## 状态机快速参考

### workflow 流转

```
init → ready → dev → archived
```

### phrases.status 流转

```
init → dev → finished
          └→ issue_found → fixing → finished
```

### fixes.status 流转

```
pending → planned → executing → done
                      └→ failed → planned
```

## 入口点

- **状态机定义**: `@project_docs/roadmap.json`
- **需求文档**: `@docs/new_spec_v2.md`
- **开发阶段**: `@project_docs/phrases/`
- **修复计划**: `@project_docs/fixes/`
- **归档目录**: `@project_docs/archives/`
