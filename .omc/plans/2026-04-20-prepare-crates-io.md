# Plan: 准备发布 ddd 到 crates.io

## Requirements Summary

修复 `exec.rs` 中的构建错误（`state.phases` → `state.phrases`），并完善 `Cargo.toml` 元数据以满足 crates.io 发布要求。

## Acceptance Criteria

1. `cargo build` 编译通过，0 errors
2. `cargo test` 所有测试通过
3. `Cargo.toml` 包含 crates.io 必需字段：description, repository, license, keywords, categories
4. 代码可正常发布到 crates.io

## Implementation Steps

### Step 1: 修复 exec.rs 的字段引用错误

**文件**: `src/commands/exec.rs`

将 `state.phases` 和 `new_state.phases` 全部替换为 `state.phrases` 和 `new_state.phrases`（共 5 处）

```diff
- state.phases.iter().find(|p| &p.name == name)
+ state.phrases.iter().find(|p| &p.name == name)

- state.phases.first()
+ state.phrases.first()

- state.phases.iter().position(...)
- idx.and_then(|i| state.phases.get(i + 1))
+ state.phrases.iter().position(...)
+ idx.and_then(|i| state.phrases.get(i + 1))

- new_state.phases.iter_mut().find(...)
+ new_state.phrases.iter_mut().find(...)
```

### Step 2: 完善 Cargo.toml 元数据

**文件**: `Cargo.toml`

添加 crates.io 必需字段：

```toml
[package]
name = "ddd"
version = "0.1.0"
edition = "2021"
description = "DocDriven CLI - 文档驱动开发框架，通过状态机管理 AI Agent 的开发阶段"
repository = "https://github.com/ipconfiger/ddd-tool"
license = "MIT"
keywords = ["ddd", "doc-driven", "cli", "agent", "roadmap"]
categories = ["command-line-utilities", "development-tools"]
authors = ["ipconfiger"]
```

### Step 3: 验证构建和测试

```bash
cargo build
cargo test
```

### Step 4: 发布前检查

- 运行 `cargo publish --dry-run` 确认无误
- 确认 version 为合适版本号（当前 0.1.0 可用）

## Verification

| 步骤 | 验证命令 | 预期结果 |
|------|----------|----------|
| 构建 | `cargo build` | 0 errors |
| 测试 | `cargo test` | All tests pass |
| 干跑 | `cargo publish --dry-run` | 无错误 |

## Risks and Mitigations

- **风险**: 字段名修改影响其他模块
  - **缓解**: grep 确认只有 exec.rs 使用 `.phases`，其他模块已用 `.phrases`

## ADR

**Decision**: 修复 exec.rs 的 `phases` → `phrases`，不重命名 `RoadmapState.phrases` 字段

**Drivers**:
- spec 文档 (`docs/new_spec_v2.md`) 使用 `phrases` 作为字段名
- `RoadmapState` 定义在 `roadmap.rs` 中使用 `phrases`
- 其他所有命令模块（verify, fix_plan, internal 等）已正确使用 `phrases`
- exec.rs 是唯一错误引用 `phases` 的文件

**Alternatives considered**:
- 重命名 `RoadmapState.phrases` → `phases`：工作量大，需要修改 roadmap.rs 和所有引用处
- 保持 exec.rs 不变：无法通过编译

**Why chosen**: 最少改动，最大一致。exec.rs 的 `phases` 明显是笔误。

**Consequences**: 编译通过，项目可发布。

**Follow-ups**: None — 此修复不引入新功能
