# PRD: Fix Rust Compiler Warnings

## Problem Statement

Rust 编译器产生 14 个警告，影响代码质量和编译输出。

## Goals

1. 消除所有 unused import 警告
2. 消除所有 unused variable 警告
3. 消除所有 dead code 警告
4. 保持功能不变

## Non-Goals

- 不重构代码结构
- 不添加新功能

## Acceptance Criteria

- [ ] `cargo build 2>&1` 输出 0 warnings
- [ ] `cargo test` 仍然 15 passed

## Implementation

### 步骤 1: 清理 unused imports

| 文件 | 修复 |
|------|------|
| src/commands/init.rs | 移除 `use std::path::Path` |
| src/commands/exec.rs | 移除 `use crate::state::RoadmapState` |
| src/commands/fix_plan.rs | 移除 `use crate::state::{Fix, Phrase}` |
| src/prompts/mod.rs | 移除未使用的 imports |

### 步骤 2: 清理 unused variables

| 文件 | 修复 |
|------|------|
| src/commands/fix_plan.rs | `_fix_id` 改为 `_` |
| src/commands/resume.rs | `_current_phase` 改为 `_` |

### 步骤 3: 处理 dead code

| 项目 | 决策 |
|------|------|
| FileLock | 添加 `#[allow(dead_code)]` |
| RoadmapStore::init | 添加 `#[allow(dead_code)]` |
| PromptParams::with_name | 添加 `#[allow(dead_code)]` |

## Verification

```bash
cargo build 2>&1 | grep -E "warning:|error:"
cargo test
```
