# PRD: Clean All Rust Warnings

## Problem Statement

`cargo build` 产生 10 个 warning,`cargo clippy --all-targets` 产生 16 个 warning(其中 6 个与 build 重叠)。覆盖死代码、未使用变量、代码质量问题。

## Baseline (2026-06-05)

- `cargo build`: 10 warnings(0 errors)
- `cargo clippy --all-targets`: 16 warnings(0 errors)
- `cargo test`: 16 passed

## Goals

1. `cargo build` 输出 0 warning
2. `cargo clippy --all-targets` 输出 0 warning
3. 行为与功能完全不变
4. 所有现有测试通过

## Non-Goals

- 不重构代码结构
- 不新增/补全功能(如实现 `backup_dir` 真实逻辑)
- 不改动公共 API 签名

## 警告清单与处理决策

### Group A: 死代码 → 删除(6 个常量 + 2 个 stub 函数)

| 位置 | 项 | 决策 | 理由 |
|------|------|------|------|
| `src/state/constants.rs:1-6` | `STATE_INIT/PREPARE/DEVELOPING/VERIFYING/FIXING/READY` | **删除整个文件** | 6 个常量全部 unused;`roadmap.rs:17-18` 已用 `WORKFLOW_STATES`/`PHASE_STATES` 内联字面量,无外部引用 |
| `src/commands/setup.rs:228` | `fn backup_dir` | **删除** | 签名带 `_` 前缀的 stub(`Ok(())`),调用点已被注释掉(`L88`, `L121`),未来需用时再实现 |
| `src/commands/setup.rs:232` | `fn backup_file` | **删除** | 同上,stub 无实现 |

### Group B: 死代码 → `#[allow(dead_code)]`(沿用既有模式)

| 位置 | 项 | 决策 | 理由 |
|------|------|------|------|
| `src/prompts/mod.rs:27` | `with_anem` | **加 `#[allow(dead_code)]`** | 测试 `L94` 使用;同 `with_name`(`L42`)既有模式 |
| `src/prompts/mod.rs:32` | `with_phase_name` | **加 `#[allow(dead_code)]`** | 测试 `L95` 使用;同 `with_name` 模式 |
| `src/prompts/mod.rs:37` | `with_plan_file` | **加 `#[allow(dead_code)]`** | 测试 `L96` 使用;同 `with_name` 模式 |

### Group C: 真实代码问题 → 修复

| 位置 | 警告 | 修复 |
|------|------|------|
| `src/commands/setup.rs:91` | unused `desc` | `for (name, _desc) in PUBLIC_COMMANDS`(`setup_claude` 内不读取 desc;`setup_opencode` 内的循环 `L124` 需保留 `desc`) |
| `src/commands/sync.rs:29-30` | unnecessary `if let` since only `Ok` variant used | 改为 `for entry in fs::read_dir(&specs_path)?.filter_map(Result::ok) { ... }` |
| `src/state/roadmap.rs:74` | needlessly taken reference of left operand | `&p.name == phase_name` → `p.name == phase_name` |
| `src/state/roadmap.rs:80` | needlessly taken reference of left operand | `&p.name == phase_name` → `p.name == phase_name` |
| `src/state/roadmap.rs:95` | creates owned `String` for comparison | `...status == "init".to_string()` → `...status == "init"` |
| `src/state/roadmap.rs:140` | `.enumerate()` index discarded | `.into_iter().enumerate().map(|(name, file)| ...)` 不行(类型不匹配);改为 `.into_iter().map(|(name, file)| ...)` 直接丢掉 `.enumerate()` |
| `src/state/roadmap.rs:191` | file opened with `create` but no `truncate` | `FileLock::lock`(`L188-193`) 链式调用中追加 `.truncate(true)`,与 `RoadmapStore::save`(`L266`) 行为一致 |

## Implementation Steps

按文件分组,自下而上修改:

### Step 1: `src/state/constants.rs`
- 删除整个文件(6 行常量)
- 检查 `src/state/mod.rs` 是否有 `mod constants;` 声明,如有同步删除

### Step 2: `src/commands/setup.rs`
- `L91`: `for (name, desc)` → `for (name, _desc)`
- `L228-234`: 删除 `backup_dir` 与 `backup_file` 两个 stub 函数

### Step 3: `src/prompts/mod.rs`
- `L27`, `L32`, `L37`: 三个 builder 方法前各加一行 `#[allow(dead_code)]`

### Step 4: `src/commands/sync.rs`
- `L29-35`: 嵌套 `if let` 改为 `.filter_map(Result::ok)` 链式
- 验证 `entry.path().extension()...` 闭包内逻辑无回归

### Step 5: `src/state/roadmap.rs`
- `L74`: `&p.name == phase_name` → `p.name == phase_name`
- `L80`: `&p.name == phase_name` → `p.name == phase_name`
- `L95`: `== "init".to_string()` → `== "init"`
- `L140`: `.into_iter().enumerate()` → `.into_iter()`
- `L191`: `.create(true)` 后追加 `.truncate(true)`

## Risks & Mitigations

| 风险 | 缓解 |
|------|------|
| 删除 `STATE_*` 常量导致外部依赖断裂 | 全文 `grep STATE_INIT\|STATE_PREPARE\|...` 确认无引用;`constants.rs` 整个文件 unused |
| `setup.rs:91` 改 `_desc` 影响 `setup_opencode` | `setup_opencode` 在 `L124` 是独立循环,作用域隔离,不受影响 |
| `roadmap.rs:191` 加 `truncate(true)` 改变锁文件语义 | `FileLock` 当前 `#[allow(dead_code)]` 未被生产路径使用,`truncate` 与 `RoadmapStore::save` 行为一致,无回归 |
| `roadmap.rs:140` 移除 `.enumerate()` 需确认下游闭包 | 当前 `.map(|(_idx, (name, file))| ...)` 直接解构,移除 `.enumerate()` 后应写为 `.map(|(name, file)| ...)` |
| 修改 `prompts/mod.rs` 破坏测试 | 测试在 `cfg(test)`,build 不编译,clippy/test 才会触发;三个 builder 在测试中被调用,加 `#[allow(dead_code)]` 与既有 `with_name` 模式一致 |

## Verification

按以下顺序执行,全部必须通过:

```bash
# 1. 编译无 warning
cargo build 2>&1 | tee /tmp/build.log
test "$(grep -c 'warning:' /tmp/build.log)" = "0"

# 2. Clippy 无 warning
cargo clippy --all-targets 2>&1 | tee /tmp/clippy.log
test "$(grep -c 'warning:' /tmp/clippy.log)" = "0"

# 3. 测试无回归
cargo test 2>&1 | tee /tmp/test.log
grep -E "test result: ok\." /tmp/test.log
# 期望: 16 passed (与 baseline 一致)
```

## Definition of Done

- [ ] `cargo build 2>&1 | grep -c warning:` = 0
- [ ] `cargo clippy --all-targets 2>&1 | grep -c warning:` = 0
- [ ] `cargo test` 通过,用例数 ≥ 16
- [ ] 修改的文件清单:
  - `src/state/constants.rs` (删除)
  - `src/state/mod.rs` (若含 `mod constants;`,删除)
  - `src/commands/setup.rs`
  - `src/prompts/mod.rs`
  - `src/commands/sync.rs`
  - `src/state/roadmap.rs`
