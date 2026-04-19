# DocDriven State Machine Specification

## 全局工作流 (workflow)

```
init ──(prepare完成)──▶ ready ──(exec触发)──▶ dev ──(archive)──▶ archived
```

### 状态说明

| 状态 | 说明 |
|------|------|
| `init` | 初始状态，等待需求输入 |
| `ready` | 文档准备就绪，等待执行 |
| `dev` | 开发中状态 |
| `archived` | 归档完成 |

## 阶段状态 (phrases[].status)

```
init ──(exec)──▶ dev ──(verify成功)──▶ finished
                      └─(verify失败)──▶ issue_found ──(fix闭环)──▶ fixing ──(verify通过)──▶ finished
```

### 状态说明

| 状态 | 说明 |
|------|------|
| `init` | 阶段初始化 |
| `dev` | 开发中 |
| `issue_found` | 发现问题，等待修复 |
| `fixing` | 修复中 |
| `finished` | 阶段完成 |

## 修复状态 (fixes[].status)

```
pending ──(plan)──▶ planned ──(exec)──▶ executing ──(验证通过)──▶ done
                                                └─(验证失败)──▶ failed ──(重新plan)──▶ planned
```

### 状态说明

| 状态 | 说明 |
|------|------|
| `pending` | 等待规划 |
| `planned` | 计划已生成 |
| `executing` | 执行中 |
| `done` | 完成 |
| `failed` | 失败，需要重新规划 |

## 状态锁机制

采用 `flock` 保护 `roadmap.json`，每次读写前执行 JSON Schema 校验，防止多 Agent 并发撕裂。

## 可观测性

每次状态变更自动追加 `history` 数组（可选），记录：
- `timestamp`: ISO8601 时间戳
- `command`: 触发命令
- `from_state`: 原状态
- `to_state`: 新状态

## 相关文档

- 核心机制：[@project_docs/specs/spec_core.md](spec_core.md)
- 命令定义：[@project_docs/specs/spec_commands.md](spec_commands.md)
- 数据流动：[@project_docs/specs/spec_data_flow.md](spec_data_flow.md)
- 工程规范：[@project_docs/specs/spec_engineering.md](spec_engineering.md)
