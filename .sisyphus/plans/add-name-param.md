# Add `name` Parameter to command_prompt / skill_prompt

## TL;DR

> **Quick Summary**: 在 `DddCommand` trait 的 `command_prompt()` 和 `skill_prompt()` 方法签名中新增 `name: &str` 参数，使 prompt 渲染时可以使用命令/技能名称。用户选择参数方式而非 `self.name()`。
>
> **Deliverables**:
> - Trait 签名变更（2 个方法）
> - 12 个 impl 的签名和 format string 更新
> - 2 个 call site 传入 `name`
> - 1 个 test mock 签名更新
>
> **Estimated Effort**: Quick（纯机械变更，14 个文件）
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Task 1 (trait def) → Task 2-5 (impls parallel) → Task 6 (call sites + mock)

---

## Context

### Original Request
用户要求："在 command_prompt 和 skill_prompt 函数的参数里除了bin之外还要加入name参数,在渲染prompt的时候需要用到"

### Interview Summary
**Key Discussions**:
- Metis 发现 trait 已有 `fn name(&self) -> &'static str`，可在 impl 内直接调用 `self.name()`
- 用户明确选择新增 `name: &str` 参数方式（而非使用 `self.name()`）

### Metis Review
**Identified Gaps** (addressed):
- `self.name()` 替代方案 → 用户选择参数方式
- prompt 中名称出现位置（frontmatter / CLI invocation / description text）→ 全部统一使用 `{name}` 替换
- 输出应保持 byte-identical → 是，纯重构

---

## Work Objectives

### Core Objective
在 `DddCommand` trait 的 `command_prompt` 和 `skill_prompt` 方法中新增 `name: &str` 参数，并在所有 prompt 模板的 format string 中使用该参数替换硬编码的命令名。

### Concrete Deliverables
- `src/commands/trait_def.rs` — 2 个方法签名新增 `name: &str`
- 11 个 impl 文件 — 各自 2 个方法签名 + format string 更新
- `src/commands/setup.rs` — 2 个调用点传入 `cmd.name()`
- `src/commands/registry.rs` — MockCommand 签名更新

### Definition of Done
- [ ] `cargo build` 零错误
- [ ] `cargo test` 全部通过
- [ ] `git diff --stat` 恰好涉及 14 个文件

### Must Have
- 所有 `command_prompt` 和 `skill_prompt` 签名新增 `name: &str`
- 所有 format string 中硬编码的子命令名替换为 `{name}`
- call site 传入 `cmd.name()`
- 编译通过 + 测试通过

### Must NOT Have (Guardrails)
- 不得修改 `execute()` 方法
- 不得修改 `prompt_template()` 常量或 `render()` 调用链
- 不得修改 `PromptTask` struct 或其 `name` 字段（setup.rs:41 已正确使用 `cmd.name()`）
- 不得改变 prompt 输出的语义内容（纯参数化重构）
- 不得引入新的 clippy warning

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** - ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (cargo test)
- **Automated tests**: Tests-after（现有测试覆盖，确保不回归）
- **Framework**: cargo test

### QA Policy
- Rust 编译验证：`cargo build 2>&1`
- 测试验证：`cargo test 2>&1`
- Clippy 检查：`cargo clippy 2>&1 | grep warning`

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (foundation - trait definition):
└── Task 1: 更新 trait_def.rs 签名 [quick]

Wave 2 (impls - MAX PARALLEL, 4 groups of 3):
├── Task 2: init + exec + verify 签名和 format string [quick]
├── Task 3: prepare + audit + final_verify 签名和 format string [quick]
├── Task 4: confirm + archive + report 签名和 format string [quick]
└── Task 5: sync + accept(internal) + mock(registry) 签名 [quick]

Wave 3 (integration):
└── Task 6: setup.rs 调用点更新 + cargo build + cargo test [quick]

Critical Path: Task 1 → Tasks 2-5 → Task 6
Parallel Speedup: ~50% faster than sequential
Max Concurrent: 4 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 | - | 2,3,4,5 |
| 2 | 1 | 6 |
| 3 | 1 | 6 |
| 4 | 1 | 6 |
| 5 | 1 | 6 |
| 6 | 2,3,4,5 | - |

### Agent Dispatch Summary

- **Wave 1**: 1 task — T1 → `quick`
- **Wave 2**: 4 tasks — T2-T5 → `quick`
- **Wave 3**: 1 task — T6 → `quick`

---

## TODOs

- [x] 1. 更新 DddCommand trait 签名

  **What to do**:
  - 打开 `src/commands/trait_def.rs`
  - 将 `fn command_prompt(&self, bin: &str) -> Option<String>` 改为 `fn command_prompt(&self, bin: &str, name: &str) -> Option<String>`
  - 将 `fn skill_prompt(&self, bin: &str) -> Option<String>` 改为 `fn skill_prompt(&self, bin: &str, name: &str) -> Option<String>`

  **Must NOT do**:
  - 不得修改其他 trait 方法（name(), description(), execute() 等）

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 (standalone)
  - **Blocks**: Tasks 2, 3, 4, 5
  - **Blocked By**: None

  **References**:
  - `src/commands/trait_def.rs` — trait 定义，需要改两行方法签名

  **Acceptance Criteria**:
  - [ ] 两个方法签名均包含 `name: &str` 参数

  **QA Scenarios**:
  ```
  Scenario: Trait 签名变更正确
    Tool: Bash (grep)
    Steps:
      1. grep -n "fn command_prompt\|fn skill_prompt" src/commands/trait_def.rs
      2. 确认两个签名都包含 `name: &str`
    Expected Result: 两个方法签名中都有 `, name: &str`
    Evidence: .sisyphus/evidence/task-1-trait-sig.txt
  ```

  **Commit**: NO (group with final)

- [x] 2. 更新 init, exec, verify 的 impl

  **What to do**:
  - 打开 `src/commands/init.rs`、`src/commands/exec.rs`、`src/commands/verify.rs`
  - 每个文件：
    1. `command_prompt` 签名新增 `name: &str`
    2. `skill_prompt` 签名新增 `name: &str`
    3. format string 中硬编码的子命令名（如 `exec`、`init`、`verify`）替换为 `{name}`
    4. 在函数体中 `name` 变量已通过参数获得，直接用于 format!()

  **Must NOT do**:
  - 不得修改 execute() 方法、prompt_template 常量、render() 调用
  - 不得改变 prompt 的语义内容

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 3, 4, 5)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `src/commands/init.rs` — InitCommand impl，当前 command_prompt 和 skill_prompt 中 `init` 为硬编码
  - `src/commands/exec.rs` — ExecCommand impl，同上，`exec` 为硬编码
  - `src/commands/verify.rs` — VerifyCommand impl，同上，`verify` 为硬编码

  **Acceptance Criteria**:
  - [ ] 3 个文件 × 2 个方法 = 6 处签名更新
  - [ ] 所有 format string 中子命令名使用 `{name}`

  **QA Scenarios**:
  ```
  Scenario: format string 使用 {name}
    Tool: Bash (grep)
    Steps:
      1. grep -n "fn command_prompt\|fn skill_prompt" src/commands/init.rs src/commands/exec.rs src/commands/verify.rs
      2. 确认签名中有 name: &str
      3. grep -A5 "fn command_prompt" 同上文件，确认 format! 中无硬编码子命令名
    Expected Result: 所有方法签名含 name 参数，format 用 {name}
    Evidence: .sisyphus/evidence/task-2-impls.txt
  ```

  **Commit**: NO (group with final)

- [x] 3. 更新 prepare, audit, final_verify 的 impl

  **What to do**:
  - 打开 `src/commands/prepare.rs`、`src/commands/audit.rs`、`src/commands/final_verify.rs`
  - 每个文件：同 Task 2 的操作步骤
    - 签名新增 `name: &str`
    - format string 中硬编码的子命令名替换为 `{name}`

  **Must NOT do**:
  - 不得修改 execute()、prompt_template()、render() 等

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 2, 4, 5)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `src/commands/prepare.rs` — PrepareCommand impl
  - `src/commands/audit.rs` — AuditCommand impl
  - `src/commands/final_verify.rs` — FinalVerifyCommand impl

  **Acceptance Criteria**:
  - [ ] 3 个文件 × 2 个方法 = 6 处签名更新

  **QA Scenarios**:
  ```
  Scenario: 签名和 format 正确
    Tool: Bash (grep)
    Steps:
      1. grep -n "name: &str" src/commands/prepare.rs src/commands/audit.rs src/commands/final_verify.rs
      2. 确认每个文件至少 2 处匹配
    Expected Result: 每个文件 2 处 name: &str
    Evidence: .sisyphus/evidence/task-3-impls.txt
  ```

  **Commit**: NO (group with final)

- [x] 4. 更新 confirm, archive, report 的 impl

  **What to do**:
  - 打开 `src/commands/confirm_phase.rs`、`src/commands/archive.rs`、`src/commands/report.rs`
  - 每个文件：同 Task 2 的操作步骤

  **Must NOT do**:
  - 不得修改 execute()、prompt_template()、render() 等

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 2, 3, 5)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `src/commands/confirm_phase.rs` — ConfirmCommand impl
  - `src/commands/archive.rs` — ArchiveCommand impl
  - `src/commands/report.rs` — ReportCommand impl

  **Acceptance Criteria**:
  - [ ] 3 个文件 × 2 个方法 = 6 处签名更新

  **QA Scenarios**:
  ```
  Scenario: 签名和 format 正确
    Tool: Bash (grep)
    Steps:
      1. grep -n "name: &str" src/commands/confirm_phase.rs src/commands/archive.rs src/commands/report.rs
      2. 确认每个文件至少 2 处匹配
    Expected Result: 每个文件 2 处 name: &str
    Evidence: .sisyphus/evidence/task-4-impls.txt
  ```

  **Commit**: NO (group with final)

- [x] 5. 更新 sync, accept(internal), mock(registry) 的 impl

  **What to do**:
  - 打开 `src/commands/sync.rs`、`src/commands/internal.rs`（AcceptCommand）、`src/commands/registry.rs`（MockCommand）
  - 每个文件：同 Task 2 的操作步骤
  - 注意 `registry.rs` 中 MockCommand 的 `_bin` 改为 `_bin`，`_name` 保持 `_` 前缀（未使用）

  **Must NOT do**:
  - 不得修改 MockCommand 的返回值内容
  - 不得修改非相关的测试代码

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 2, 3, 4)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `src/commands/sync.rs` — SyncCommand impl
  - `src/commands/internal.rs` — AcceptCommand impl
  - `src/commands/registry.rs` — MockCommand test impl（第 30-50 行左右）

  **Acceptance Criteria**:
  - [ ] 3 个文件 × 2 个方法 = 6 处签名更新
  - [ ] MockCommand 的 `name` 参数用 `_name` 前缀（避免 unused warning）

  **QA Scenarios**:
  ```
  Scenario: mock 签名正确且无 unused warning
    Tool: Bash (grep)
    Steps:
      1. grep -n "_name: &str" src/commands/registry.rs
      2. 确认存在匹配
    Expected Result: MockCommand 方法签名含 _name: &str
    Evidence: .sisyphus/evidence/task-5-mock.txt
  ```

  **Commit**: NO (group with final)

- [x] 6. 更新 call sites + 编译测试验证

  **What to do**:
  - 打开 `src/commands/setup.rs`
  - 找到 `cmd.command_prompt(...)` 调用（约 line 70），将 `cmd.command_prompt(ddd_binary.to_string_lossy().as_ref())` 改为 `cmd.command_prompt(ddd_binary.to_string_lossy().as_ref(), cmd.name())`
  - 找到 `cmd.skill_prompt(...)` 调用（约 line 134），将 `cmd.skill_prompt(ddd_binary.to_string_lossy().as_ref())` 改为 `cmd.skill_prompt(ddd_binary.to_string_lossy().as_ref(), cmd.name())`
  - 运行 `cargo build 2>&1` 确认编译通过
  - 运行 `cargo test 2>&1` 确认测试通过
  - 运行 `cargo clippy 2>&1` 确认无新增 warning

  **Must NOT do**:
  - 不得修改 PromptTask struct 的 name 字段（已正确使用 cmd.name()）
  - 不得修改 setup_opencode() 函数中其他逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (standalone, after Wave 2)
  - **Blocks**: Final Verification
  - **Blocked By**: Tasks 2, 3, 4, 5

  **References**:
  - `src/commands/setup.rs:68-70` — command_prompt call site，`cmd.name()` 在 line 68 已可用
  - `src/commands/setup.rs:112-134` — skill_prompt call site，`cmd.name()` 在 line 112 已可用

  **Acceptance Criteria**:
  - [ ] 2 个调用点传入 `cmd.name()` 作为第二参数
  - [ ] `cargo build` 零错误
  - [ ] `cargo test` 全部通过
  - [ ] `cargo clippy` 无新增 warning

  **QA Scenarios**:
  ```
  Scenario: 编译通过
    Tool: Bash
    Steps:
      1. cargo build 2>&1
      2. 检查 exit code = 0
    Expected Result: Build succeeded, 0 errors
    Evidence: .sisyphus/evidence/task-6-build.txt

  Scenario: 测试通过
    Tool: Bash
    Steps:
      1. cargo test 2>&1
      2. 检查 "test result: ok"
    Expected Result: All tests pass
    Evidence: .sisyphus/evidence/task-6-test.txt

  Scenario: Clippy 无新增 warning
    Tool: Bash
    Steps:
      1. cargo clippy 2>&1
      2. grep "warning" 检查无新增
    Expected Result: No new warnings
    Evidence: .sisyphus/evidence/task-6-clippy.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): add name param to command_prompt/skill_prompt trait methods`
  - Files: All 14 files
  - Pre-commit: `cargo build && cargo test`

---

## Final Verification Wave

> After ALL implementation tasks, run verification.

- [x] F1. **Build Verification** — `cargo build 2>&1` 零错误 + `cargo test 2>&1` 全通过 + `cargo clippy 2>&1` 无新增 warning
- [x] F2. **Diff Audit** — `git diff --stat` 确认恰好 14 个文件变更，无多余修改

## Commit Strategy

- **Single Commit**: `refactor(commands): add name param to command_prompt/skill_prompt trait methods`
  - All 14 files
  - Pre-commit: `cargo build && cargo test`

## Success Criteria

### Verification Commands
```bash
cargo build 2>&1   # Expected: zero errors
cargo test 2>&1    # Expected: all tests pass
cargo clippy 2>&1  # Expected: no new warnings
git diff --stat    # Expected: 14 files changed
```

### Final Checklist
- [ ] All `command_prompt` / `skill_prompt` signatures have `name: &str`
- [ ] All format strings use `{name}` for subcommand name
- [ ] Call sites pass `cmd.name()`
- [ ] `cargo build` passes
- [ ] `cargo test` passes
