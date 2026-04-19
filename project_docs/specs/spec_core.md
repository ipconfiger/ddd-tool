# DocDriven Core Specification

## 概述

DocDriven 是一个文档驱动的开发框架，通过状态机和 CLI 工具锁定的机制，将开发者的工作限定在围绕文档的规范中。

## 核心机制

### 技术选型

| 项目 | 技术 |
|-----|------|
| **语言** | Rust |
| **运行环境** | CLI 二进制，集成至 `opencode` / `claude code` 的 `Skills` 与 `Commands` 生态 |

### 状态机定义

状态机文件：`@project_docs/roadmap.json`

采用 **全局工作流与局部阶段指针分离设计**，字段语义严格隔离。

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

### 核心概念

- **workflow**: 全局工作流状态，控制主流程
- **current_phase**: 指向当前阶段的指针
- **doc_ready**: 文档准备就绪标志
- **phrases**: 阶段数组，每个阶段包含独立的 fixes 修复记录

## 相关文档

- 命令定义：[@project_docs/specs/spec_commands.md](spec_commands.md)
- 数据流动：[@project_docs/specs/spec_data_flow.md](spec_data_flow.md)
- 状态流转：[@project_docs/specs/spec_state_machine.md](spec_state_machine.md)
- 工程规范：[@project_docs/specs/spec_engineering.md](spec_engineering.md)
