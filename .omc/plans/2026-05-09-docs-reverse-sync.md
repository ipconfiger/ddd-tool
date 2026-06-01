# 文档反向同步计划

## 任务概述

根据当前代码实现，反向更新所有文档，确保文档与代码一致。

## 发现的主要差异

### 1. 命令差异

| 类别 | 文档有但代码无 | 代码有但文档无 |
|------|---------------|---------------|
| 核心命令 | `fix-plan`, `fix-exec` | `audit`, `confirm`, `final`, `accept` |
| 内部触发器 | `gen-phase`, `set-issuse`, `finish-fix`, `finish-phase` | (同上) |
| 辅助命令 | `resume`, `help`, `version`, `study` | — |

### 2. 状态机差异

- **代码有但文档无:** `verifying` 状态 (verify.rs:48)
- **文档有但代码无:** `issue_found`, `fixing` 状态
- **代码验证的状态:** `["init", "dev", "verifying", "finished"]` (roadmap.rs:17-18)

### 3. 目录名不一致

- `prepare.rs` → `project_docs/phases/`
- `internal.rs` → `project_docs/phases/` (注意 'a')
- 文档混用 `phases/` 和 `phases/`

### 4. Prompt 差异

- `init`: 文档说用 `@Plan` agent，实际是 MVP/KISS 原则
- `prepare`: 文档说调用 `!ddd gen_phase`，实际提示 `/ddd-accept`
- `verify`: 文档说调用 `finish_phase`，实际提示 `/ddd-confirm`
- `exec`: 文档说建议 `/ddd-exec`，实际建议 `/ddd-confirm`

---

## 更新计划

### 步骤 1: 更新 SPEC_CORE (spec_core.md)

**文件:** `project_docs/specs/spec_core.md`

更新内容:
- [ ] 确认 `init`, `prepare`, `exec`, `verify`, `archive`, `report`, `sync` 命令列表
- [ ] 补充 `audit`, `confirm`, `final`, `accept` 命令
- [ ] 移除不存在的命令: `fix-plan`, `fix-exec`, `gen-phase`, `set-issuse`, `finish-fix`, `finish-phase`, `resume`, `help`, `version`, `study`

### 步骤 2: 更新 SPEC_COMMANDS (spec_commands.md)

**文件:** `project_docs/specs/spec_commands.md`

更新内容:
- [ ] 更新所有命令描述和 prompt 为实际代码内容
- [ ] `init` prompt 改为 MVP/KISS 版本
- [ ] `prepare` 改为提示 `/ddd-accept`
- [ ] `verify` 改为提示 `/ddd-confirm`
- [ ] `exec` 改为建议 `/ddd-confirm`
- [ ] 添加 `audit`, `confirm`, `final`, `accept` 命令定义
- [ ] 移除 `fix-plan`, `fix-exec`, `gen-phase` 等未实现命令
- [ ] 统一目录名为 `phases/` 或 `phases/` (需决定统一哪个)

### 步骤 3: 更新 SPEC_STATE_MACHINE (spec_state_machine.md)

**文件:** `project_docs/specs/spec_state_machine.md`

更新内容:
- [ ] 阶段状态改为: `["init", "dev", "verifying", "finished"]`
- [ ] 移除 `issue_found`, `fixing` 状态
- [ ] 添加 `verifying` 状态说明
- [ ] 更新 Fix 状态流程 (或标记为未实现)

### 步骤 4: 更新 SPEC_DATA_FLOW (spec_data_flow.md)

**文件:** `project_docs/specs/spec_data_flow.md`

更新内容:
- [ ] 统一目录引用为 `phases/` 或 `phases/`
- [ ] 更新 prompt 参数映射

### 步骤 5: 更新 NEW_SPEC_V2 (docs/new_spec_v2.md)

**文件:** `docs/new_spec_v2.md`

更新内容:
- [ ] 同步所有命令变更
- [ ] 更新状态机定义
- [ ] 更新 prompt 内容
- [ ] 添加新命令描述

### 步骤 6: 更新 OPENCODE 命令文档

**文件:** `.opencode/commands/*.md`

这些文件目前只是代理到 CLI，需要检查是否需要补充命令描述。

### 步骤 7: 更新 README.md (如需要)

**文件:** `README.md`

检查并更新项目描述是否与当前实现一致。

---

## 待确认问题

1. **目录名统一:** `phases/` 还是 `phases/`？
   - `prepare.rs` 使用 `phases/`
   - `internal.rs` 使用 `phases/`
   - 建议统一为 `phases/` (更常用)

2. **Fix 状态流程:** 文档中的 fix-plan/fix-exec 流程是应该实现还是从文档移除？

3. **内部触发器:** `set-issuse`, `finish_fix` 等是否应该实现？

---

## 验收标准

- [ ] `spec_commands.md` 与实际命令一致
- [ ] `spec_state_machine.md` 与 roadmap.rs 中的状态一致
- [ ] 所有命令的 prompt 与代码中的实际 prompt 一致
- [ ] 目录引用统一
- [ ] 新增命令 (audit, confirm, final, accept) 已添加到相关文档

## 涉及文件

- `project_docs/specs/spec_core.md`
- `project_docs/specs/spec_commands.md`
- `project_docs/specs/spec_state_machine.md`
- `project_docs/specs/spec_data_flow.md`
- `docs/new_spec_v2.md`
- `.opencode/commands/*.md`
- `README.md`
