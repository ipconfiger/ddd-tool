# Command Structure Refactoring — Trait-Based Architecture

## TL;DR

> **Quick Summary**: 将 11 个分散的 CLI 子命令重构为统一的 `DddCommand` trait 体系，用 trait object 替代 enum dispatch，自动注册机制替代手动维护的 PUBLIC_COMMANDS，增强 PromptParams 渲染校验。
> 
> **Deliverables**:
> - `DddCommand` trait 定义 + `CommandResult` 统一返回结构
> - `CommandRegistry` 自动注册/查找机制
> - 所有 11 个子命令的 trait 实现（迁移到统一接口）
> - `setup.rs` 改为从 registry 自动遍历生成 Skills/Commands
> - `PromptParams.render()` 增强必填参数校验 + `anem` typo 修复
> - `src/lib.rs` 创建以支持集成测试
> - 全量 TDD 测试覆盖
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: T1(lib.rs) → T2(trait+result+registry) → T4-T14(11 commands) → T15(dispatch rewrite) → T16(setup) → T3(render) → FINAL

---

## Context

### Original Request
每个子命令一个命令的结构，将对应的 Skill 的 prompt、自定义子命令的 prompt、每个子命令的返回都放到统一的结构里。setup 子命令涉及到生成 Skills 和 Command 的部分代码也要根据对应的变更修改。

### Interview Summary
**Key Discussions**:
- Prompt 存放方式: 方案 A — const 留在各命令文件中，作为 trait 关联常量
- 返回结构: CommandResult { success: bool, message: String, prompt: Option<String> }
- Setup 生成: 自动遍历注册表，每个命令自带 skill_prompt()/command_prompt()
- Dispatch: trait object (dyn DddCommand) 完全替代 Command enum
- render 增强: 校验必填参数，未传值时 panic/Err
- 测试策略: TDD

**Research Findings**:
- tokio 是依赖但完全未使用（async 不在本次范围）
- 只有 init 命令实际读取 clap 参数，其余 10 个全部忽略
- Accept 命令有 `let _ =` 丢弃错误的行为需精确保留
- setup.rs 用 `println!("Error:")` 而非 `eprintln!("错误:")` — 需精确保留
- final_verify.rs 有注释掉的 `ctx.save_state()` — 不修复
- roadmap.rs 有空测试函数 — 不修复
- 无 lib.rs — 需创建以支持 trait/registry 的集成测试

### Metis Review
**Identified Gaps** (addressed):
- 缺少 lib.rs 的决策 → 默认创建（TDD 需要）
- Accept 的 `let _ =` 错误丢弃行为 → 精确保留为 guardrail
- setup.rs 的 println vs eprintln 不一致 → 精确保留为 guardrail
- tokio 未使用 → 不移除，作为 scope creep guardrail
- 只有 init 使用 clap 参数 → trait execute 简化设计

---

## Work Objectives

### Core Objective
将 11 个 CLI 子命令从独立的 `do_run()` 函数重构为统一 `DddCommand` trait 体系，实现自动注册、统一返回、增强校验。

### Concrete Deliverables
- `src/lib.rs` — 新建，导出核心类型
- `src/commands/registry.rs` — CommandRegistry 实现
- `src/commands/trait.rs` — DddCommand trait + CommandResult 定义
- 11 个命令文件的 trait 实现（就地修改）
- `src/commands/mod.rs` — dispatch 改为 registry lookup
- `src/commands/setup.rs` — 自动遍历 registry
- `src/prompts/mod.rs` — render 增强 + typo 修复

### Definition of Done
- [ ] `cargo build` 零错误
- [ ] `cargo test` 全部通过（含新增 trait/registry/render 测试）
- [ ] `cargo clippy` 无新增 warning
- [ ] 所有 11 个子命令的 CLI 行为与重构前一致
- [ ] `ddd setup --tool claude` 和 `ddd setup --tool opencode` 生成结果与重构前一致

### Must Have
- DddCommand trait 统一接口
- CommandResult 结构化返回
- CommandRegistry 自动注册
- setup.rs 从 registry 自动生成 Skills/Commands
- PromptParams.render() 必填参数校验
- anem typo 修复
- lib.rs 创建
- TDD：先写测试再写实现

### Must NOT Have (Guardrails)
- ❌ 不改变 RoadmapState / Phase / Fix 数据结构
- ❌ 不改变状态机逻辑（workflow: init→ready→dev→archived）
- ❌ 不改变 archive 的打包逻辑
- ❌ 不移除 tokio 依赖
- ❌ 不修复 final_verify.rs 中注释掉的 `ctx.save_state()`
- ❌ 不修复 roadmap.rs 中的空测试函数
- ❌ 不改变 Accept 命令的错误丢弃行为（`let _ =`）
- ❌ 不统一 setup.rs 的 println/eprintln 风格差异
- ❌ 不添加 async（项目无 async 代码）
- ❌ 不过度抽象：trait 方法保持最小集

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** - ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (Rust #[cfg(test)] inline tests)
- **Automated tests**: TDD
- **Framework**: cargo test (standard Rust test harness)
- **lib.rs created**: YES (enables integration test imports)

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Library/Module**: Use Bash — `cargo test`, `cargo build`, `cargo clippy`
- **CLI**: Use Bash — run `ddd` commands, assert output and exit code

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — foundation):
├── T1:  Create src/lib.rs [quick]
├── T2:  Define DddCommand trait + CommandResult + CommandRegistry [deep]
└── T3:  Enhance PromptParams.render() + fix anem typo [quick]

Wave 2 (After Wave 1 — migrate all commands, MAX PARALLEL):
├── T4:  Migrate init command [quick]
├── T5:  Migrate prepare command [quick]
├── T6:  Migrate exec command [quick]
├── T7:  Migrate verify command [quick]
├── T8:  Migrate audit command [quick]
├── T9:  Migrate final_verify command [quick]
├── T10: Migrate confirm_phase command [quick]
├── T11: Migrate archive command [unspecified-high]
├── T12: Migrate report command [quick]
├── T13: Migrate sync command [quick]
├── T14: Migrate accept (internal) command [quick]

Wave 3 (After Wave 2 — integration):
├── T15: Rewrite dispatch in mod.rs to use registry [deep]
└── T16: Rewrite setup.rs to auto-traverse registry [deep]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real manual QA (unspecified-high)
└── F4: Scope fidelity check (deep)
→ Present results → Get explicit user okay

Critical Path: T1 → T2 → T4-T14 → T15 → T16 → FINAL
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 11 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| T1   | -         | T2, T4-T14 | 1 |
| T2   | T1        | T4-T14, T15, T16 | 1 |
| T3   | -         | T15 | 1 |
| T4-T14 | T1, T2  | T15, T16 | 2 |
| T15  | T3, T4-T14 | T16, FINAL | 3 |
| T16  | T4-T14, T15 | FINAL | 3 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `quick`, T2 → `deep`, T3 → `quick`
- **Wave 2**: 11 tasks — T4-T10, T12-T14 → `quick`, T11 → `unspecified-high`
- **Wave 3**: 2 tasks — T15 → `deep`, T16 → `deep`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. 创建 src/lib.rs 支持集成测试

  **What to do**:
  - 创建 `src/lib.rs`，导出 `commands`、`prompts`、`state` 模块
  - 确保 `cargo test --lib` 能运行（lib.rs 中的 inline tests）
  - 确保 `cargo build` 同时编译 lib 和 bin

  **Must NOT do**:
  - 不改变 main.rs 的功能，只是让 lib 目标存在
  - 不改变任何现有模块的逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T2, T3)
  - **Blocks**: T2
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src/main.rs` — 当前入口，lib.rs 需要的模块路径从这里推断

  **External References**:
  - Rust Package with both lib.rs and main.rs: https://doc.rust-lang.org/cargo/guide/project-layout.html

  **Acceptance Criteria**:
  - [ ] `src/lib.rs` 存在，导出 commands/prompts/state 模块
  - [ ] `cargo build` 成功
  - [ ] `cargo test --lib` 成功运行

  **QA Scenarios**:

  ```
  Scenario: lib.rs 编译通过
    Tool: Bash
    Steps:
      1. cargo build
      2. Assert exit code 0
    Expected Result: "Compiling ddd v..." + "Finished" without errors
    Evidence: .sisyphus/evidence/task-1-lib-build.txt

  Scenario: lib test 可运行
    Tool: Bash
    Steps:
      1. cargo test --lib
      2. Assert exit code 0
    Expected Result: "running N tests" + "test result: ok"
    Evidence: .sisyphus/evidence/task-1-lib-test.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): create lib.rs for integration test support`
  - Files: `src/lib.rs`

- [x] 2. 定义 DddCommand trait + CommandResult + CommandRegistry

  **What to do**:
  - 创建 `src/commands/trait.rs`，定义:
    ```rust
    pub struct CommandResult {
        pub success: bool,
        pub message: String,
        pub prompt: Option<String>,
    }
    
    pub trait DddCommand: Send + Sync {
        fn name(&self) -> &'static str;
        fn description(&self) -> &'static str;
        fn prompt_template(&self) -> Option<&'static str>;
        fn required_params(&self) -> Vec<&'static str> { vec![] }
        fn execute(&self, ctx: &DddContext, args: &str) -> Result<CommandResult>;
        fn skill_prompt(&self) -> Option<String> { None }
        fn command_prompt(&self, bin: &str) -> Option<String> { None }
    }
    ```
  - 创建 `src/commands/registry.rs`，定义:
    ```rust
    pub struct CommandRegistry {
        commands: HashMap<&'static str, Box<dyn DddCommand>>,
    }
    ```
    - `new()` — 注册所有命令
    - `get(name) -> Option<&dyn DddCommand>`
    - `all() -> Vec<&dyn DddCommand>` — setup 用
    - `names() -> Vec<&'static str>` — 帮助信息用
  - **TDD**: 先写 trait/registry 的测试，再写实现
  - 测试覆盖: registry 注册/查找/遍历、CommandResult 构建

  **Must NOT do**:
  - 不把 execute 设计为 async
  - 不过度设计 trait 方法 — 只包含当前需要的方法
  - 不在 registry 中使用 lazy_static/once_cell 等外部依赖（用简单的 fn new()）

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []
  - Reason: 这是整个重构的基石，需要精心设计接口

  **Parallelization**:
  - **Can Run In Parallel**: YES (与 T3 并行)
  - **Parallel Group**: Wave 1 (with T1, T3)
  - **Blocks**: T4-T14, T15, T16
  - **Blocked By**: T1

  **References**:

  **Pattern References**:
  - `src/commands/mod.rs:Command` — 当前 enum 定义，trait 需要覆盖其所有 variants
  - `src/commands/context.rs:DddContext` — execute 的 ctx 参数类型
  - `src/commands/setup.rs:PUBLIC_COMMANDS` — registry 的 all() 需要返回等价的数据
  - `src/commands/setup.rs:make_prompt()` — command_prompt() 需要复制此逻辑
  - `src/commands/setup.rs:PromptTask` — Claude command 生成的 JSON 结构

  **API/Type References**:
  - `src/state/roadmap.rs:RoadmapState` — execute 内部需要操作的状态类型
  - `src/prompts/mod.rs:PromptParams` — prompt_template 配合使用

  **Acceptance Criteria**:
  - [ ] `src/commands/trait.rs` 定义 DddCommand trait 和 CommandResult
  - [ ] `src/commands/registry.rs` 定义 CommandRegistry
  - [ ] `cargo test` 通过，包含 registry 注册/查找/遍历测试
  - [ ] `cargo clippy` 无 warning

  **QA Scenarios**:

  ```
  Scenario: Registry 注册和查找
    Tool: Bash
    Steps:
      1. cargo test --lib trait --nocapture
      2. Assert "test result: ok"
    Expected Result: All trait/registry tests pass
    Evidence: .sisyphus/evidence/task-2-trait-tests.txt

  Scenario: 编译检查
    Tool: Bash
    Steps:
      1. cargo build
      2. Assert exit code 0
    Expected Result: 编译成功无错误
    Evidence: .sisyphus/evidence/task-2-build.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): define DddCommand trait, CommandResult, and CommandRegistry`
  - Files: `src/commands/trait.rs, src/commands/registry.rs`

- [x] 3. 增强 PromptParams.render() + 修复 anem typo

  **What to do**:
  - 将 `src/prompts/mod.rs` 中 `PromptParams` 的 `anem` 字段重命名为 `name`
  - 增强 `render()` 方法:
    - 检查模板中使用的占位符是否都有对应的值
    - 如果模板有 `{context}` 但 context 为 None → 返回 `Err("Missing required parameter: context")`
    - 如果传了值但模板没有对应占位符 → 忽略（正常）
  - 更新所有使用 `PromptParams` 的地方（init, prepare, exec, verify, final_verify）适配新字段名
  - **TDD**: 先写 render 校验的测试（缺少必填参数应报错），再改实现
  - 更新现有测试适配新接口

  **Must NOT do**:
  - 不改变模板语法（仍然是 `{placeholder}` 格式）
  - 不引入外部模板引擎
  - 不改变 PromptParams 的 builder pattern

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with T1, T2)
  - **Blocks**: T15
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src/prompts/mod.rs:PromptParams` — 当前定义，有 anem typo 和简单 replace
  - `src/prompts/mod.rs:render()` — 需增强的方法
  - `src/prompts/mod.rs:tests` — 现有测试需要更新

  **API/Type References**:
  - `src/commands/init.rs` — 使用 `.anem("xxx")` 的地方，改为 `.name("xxx")`
  - `src/commands/exec.rs` — 使用 `.anem("xxx")` 的地方
  - `src/commands/verify.rs` — 使用 `.anem("xxx")` 的地方
  - `src/commands/final_verify.rs` — 使用 `.anem("xxx")` 的地方
  - `src/commands/prepare.rs` — 使用 PromptParams 的地方

  **Acceptance Criteria**:
  - [ ] `anem` 字段已重命名为 `name`
  - [ ] `render()` 在缺少必填占位符时返回 Err
  - [ ] 所有使用 PromptParams 的命令已更新字段名
  - [ ] `cargo test` 全部通过
  - [ ] `cargo clippy` 无 warning

  **QA Scenarios**:

  ```
  Scenario: render 校验缺少必填参数
    Tool: Bash
    Steps:
      1. cargo test --lib prompts --nocapture
      2. Assert "test result: ok"
    Expected Result: 测试验证: 模板有 {context} 但未传值 → Err, 所有值都传 → Ok
    Evidence: .sisyphus/evidence/task-3-render-tests.txt

  Scenario: 现有命令不受影响
    Tool: Bash
    Steps:
      1. cargo build
      2. Assert exit code 0
    Expected Result: 编译成功，所有使用处已更新
    Evidence: .sisyphus/evidence/task-3-build.txt
  ```

  **Commit**: YES
  - Message: `fix(prompts): enhance render validation and fix anem typo`
  - Files: `src/prompts/mod.rs, src/commands/init.rs, src/commands/prepare.rs, src/commands/exec.rs, src/commands/verify.rs, src/commands/final_verify.rs`

- [x] 4. 迁移 init 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/init.rs` 中定义 `pub struct InitCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"init"`
    - `description()` → `"初始化项目规格"` (从 PUBLIC_COMMANDS 取)
    - `prompt_template()` → `Some(INIT_PROMPT)` (保留现有 const)
    - `required_params()` → `vec!["context"]`
    - `execute()` → 封装现有 `do_run()` 逻辑，返回 `CommandResult`
    - `command_prompt(bin)` → `Some(format!("使用 Bash工具 执行: {bin} init $ARGUMENTS..."))`
    - `skill_prompt()` → init 命令的 skill 描述（参考 setup.rs 现有逻辑）
  - 保留现有 `do_run()` 中的状态校验（workflow=="init"）和文件读取逻辑
  - `execute()` 接收 `args: &str` 解析为 context 路径（init 是唯一用 clap 参数的命令）
  - **TDD**: 先写测试验证 trait 方法返回值

  **Must NOT do**:
  - 不改变 init 的状态校验逻辑
  - 不改变 INIT_PROMPT 的内容

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with T5-T14)
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:

  **Pattern References**:
  - `src/commands/init.rs:INIT_PROMPT` — 当前 prompt const，迁移为 trait 关联
  - `src/commands/init.rs:do_run()` — 当前执行逻辑，封装到 execute()
  - `src/commands/setup.rs:PUBLIC_COMMANDS` — init 的名称和描述

  **API/Type References**:
  - `src/commands/trait.rs:DddCommand` — 要实现的 trait
  - `src/commands/context.rs:DddContext` — execute 的 ctx 参数

  **Acceptance Criteria**:
  - [ ] `InitCommand` struct 实现 `DddCommand` trait
  - [ ] `execute()` 返回 `CommandResult` 而非 `println!`
  - [ ] `cargo test` 通过
  - [ ] `cargo clippy` 无 warning

  **QA Scenarios**:

  ```
  Scenario: InitCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib init_command --nocapture
      2. Assert "test result: ok"
    Expected Result: name()=="init", prompt_template()==Some(...), execute 返回正确 CommandResult
    Evidence: .sisyphus/evidence/task-4-init-trait.txt

  Scenario: 编译通过
    Tool: Bash
    Steps:
      1. cargo build
      2. Assert exit code 0
    Evidence: .sisyphus/evidence/task-4-build.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate init to DddCommand trait`
  - Files: `src/commands/init.rs`

- [x] 5. 迁移 prepare 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/prepare.rs` 中定义 `pub struct PrepareCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"prepare"`
    - `description()` → `"准备阶段计划"`
    - `prompt_template()` → `Some(PREPARE_PROMPT)`
    - `required_params()` → `vec!["context"]`
    - `execute()` → 封装现有 `do_run()` 逻辑（校验 workflow + 清除 phases + 渲染 prompt）
    - `command_prompt(bin)` 和 `skill_prompt()` 同 T4 模式
  - **TDD**: 先写测试

  **Must NOT do**:
  - 不改变状态校验和 phases 清除逻辑
  - 不改变 PREPARE_PROMPT 内容

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with T4, T6-T14)
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/prepare.rs:PREPARE_PROMPT` — 迁移为 trait 关联
  - `src/commands/prepare.rs:do_run()` — 封装到 execute()
  - `src/commands/setup.rs:PUBLIC_COMMANDS` — 名称和描述

  **Acceptance Criteria**:
  - [ ] `PrepareCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: PrepareCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib prepare_command --nocapture
      2. Assert "test result: ok"
    Evidence: .sisyphus/evidence/task-5-prepare-trait.txt

  Scenario: 编译通过
    Tool: Bash
    Steps:
      1. cargo build
      2. Assert exit code 0
    Evidence: .sisyphus/evidence/task-5-build.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate prepare to DddCommand trait`
  - Files: `src/commands/prepare.rs`

- [x] 6. 迁移 exec 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/exec.rs` 中定义 `pub struct ExecCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"exec"`
    - `description()` → `"执行当前阶段"`
    - `prompt_template()` → `Some(EXEC_PROMPT)`
    - `required_params()` → `vec!["file", "name"]`
    - `execute()` → 封装 do_run()（检查 doc_ready + 获取当前 phase + set_phase_dev + 渲染 + 判断是否全部完成）
    - 全部完成时的 final review message 放入 CommandResult.message
  - **TDD**: 先写测试

  **Must NOT do**:
  - 不改变 doc_ready 检查和 phase 状态推进逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/exec.rs:EXEC_PROMPT` — prompt const
  - `src/commands/exec.rs:do_run()` — 执行逻辑
  - `src/commands/setup.rs:PUBLIC_COMMANDS` — 名称和描述

  **Acceptance Criteria**:
  - [ ] `ExecCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: ExecCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib exec_command --nocapture
      2. Assert "test result: ok"
    Evidence: .sisyphus/evidence/task-6-exec-trait.txt

  Scenario: 编译通过
    Tool: Bash
    Steps:
      1. cargo build
    Evidence: .sisyphus/evidence/task-6-build.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate exec to DddCommand trait`
  - Files: `src/commands/exec.rs`

- [x] 7. 迁移 verify 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/verify.rs` 中定义 `pub struct VerifyCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"verify"`
    - `description()` → `"验证当前阶段"`
    - `prompt_template()` → `Some(VERIFY_PROMPT)`
    - `required_params()` → `vec!["file", "name"]`
    - `execute()` → 封装 do_run()（获取当前 phase + 设置 verifying + 渲染 prompt）
  - **TDD**

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/verify.rs:VERIFY_PROMPT`
  - `src/commands/verify.rs:do_run()`

  **Acceptance Criteria**:
  - [ ] `VerifyCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: VerifyCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib verify_command --nocapture
    Evidence: .sisyphus/evidence/task-7-verify-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate verify to DddCommand trait`
  - Files: `src/commands/verify.rs`

- [x] 8. 迁移 audit 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/audit.rs` 中定义 `pub struct AuditCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"audit"`
    - `description()` → `"审计规格质量"`
    - `prompt_template()` → `Some(AUDIT_PROMPT)`
    - `required_params()` → `vec![]` (audit 不需要参数，直接用原始 prompt)
    - `execute()` → 封装 do_run()（检查 specs dir + 返回 prompt）
    - **注意**: audit 当前不使用 render()，直接用 AUDIT_PROMPT。execute 中直接将 prompt 放入 CommandResult.prompt

  **Must NOT do**:
  - 不删除 audit.rs 中目前 dead_code 的 render() 函数（那是清理范畴，不在本次重构中）

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/audit.rs:AUDIT_PROMPT`
  - `src/commands/audit.rs:do_run()`

  **Acceptance Criteria**:
  - [ ] `AuditCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: AuditCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib audit_command --nocapture
    Evidence: .sisyphus/evidence/task-8-audit-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate audit to DddCommand trait`
  - Files: `src/commands/audit.rs`

- [x] 9. 迁移 final_verify 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/final_verify.rs` 中定义 `pub struct FinalVerifyCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"final"`
    - `description()` → `"最终验证所有阶段"`
    - `prompt_template()` → `Some(VERIFY_PROMPT)` (final_verify 中的 VERIFY_PROMPT)
    - `required_params()` → `vec![]`
    - `execute()` → 封装 do_run()（检查所有阶段完成 + 渲染 prompt）
  - **TDD**

  **Must NOT do**:
  - 不取消注释 `ctx.save_state()` (pre-existing behavior)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/final_verify.rs:VERIFY_PROMPT`
  - `src/commands/final_verify.rs:do_run()`

  **Acceptance Criteria**:
  - [ ] `FinalVerifyCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: FinalVerifyCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib final_verify_command --nocapture
    Evidence: .sisyphus/evidence/task-9-final-verify-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate final_verify to DddCommand trait`
  - Files: `src/commands/final_verify.rs`

- [x] 10. 迁移 confirm_phase 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/confirm_phase.rs` 中定义 `pub struct ConfirmCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"confirm"`
    - `description()` → `"确认当前阶段完成"`
    - `prompt_template()` → `None` (confirm 无 prompt)
    - `required_params()` → `vec![]`
    - `execute()` → 封装 do_run()（推进阶段状态机 + 返回 message）
    - `command_prompt(bin)` → 有（setup 中有 confirm 的命令生成）
    - `skill_prompt()` → 有
  - **TDD**

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/confirm_phase.rs:do_run()` — 阶段推进逻辑

  **Acceptance Criteria**:
  - [ ] `ConfirmCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: ConfirmCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib confirm_command --nocapture
    Evidence: .sisyphus/evidence/task-10-confirm-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate confirm_phase to DddCommand trait`
  - Files: `src/commands/confirm_phase.rs`

- [x] 11. 迁移 archive 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/archive.rs` 中定义 `pub struct ArchiveCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"archive"`
    - `description()` → `"归档项目"`
    - `prompt_template()` → `None`
    - `required_params()` → `vec![]`
    - `execute()` → 封装 do_run()（验证所有阶段完成 + 创建归档 + 清理 + 重置状态）
  - **TDD**: 保留现有 7 个 unit test，确保全部通过
  - **注意**: archive 有最多的逻辑和测试，是最复杂的无 prompt 命令

  **Must NOT do**:
  - 不改变打包逻辑（tar.gz 格式、目录结构）
  - 不改变状态重置逻辑
  - 不删除现有测试

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
  - Reason: archive 有最多逻辑和测试，需要更仔细处理

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/archive.rs:do_run()` — 完整的归档逻辑
  - `src/commands/archive.rs:archive_dirs()` — 辅助函数
  - `src/commands/archive.rs:tests` — 7 个现有测试

  **Acceptance Criteria**:
  - [ ] `ArchiveCommand` 实现 `DddCommand` trait
  - [ ] 所有现有 7 个 unit test 通过
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: ArchiveCommand 所有测试通过
    Tool: Bash
    Steps:
      1. cargo test --lib archive --nocapture
      2. Assert "7 passed, 0 failed"
    Evidence: .sisyphus/evidence/task-11-archive-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate archive to DddCommand trait`
  - Files: `src/commands/archive.rs`

- [x] 12. 迁移 report 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/report.rs` 中定义 `pub struct ReportCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"report"`
    - `description()` → `"生成项目报告"`
    - `prompt_template()` → `None`
    - `required_params()` → `vec![]`
    - `execute()` → 封装 do_run()（生成 markdown 报告 + 写入文件 + 返回 message）
  - **TDD**

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/report.rs:do_run()` — 报告生成逻辑

  **Acceptance Criteria**:
  - [ ] `ReportCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: ReportCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib report_command --nocapture
    Evidence: .sisyphus/evidence/task-12-report-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate report to DddCommand trait`
  - Files: `src/commands/report.rs`

- [x] 13. 迁移 sync 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/sync.rs` 中定义 `pub struct SyncCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"sync"`
    - `description()` → `"同步项目状态"`
    - `prompt_template()` → `None`
    - `required_params()` → `vec![]`
    - `execute()` → 封装 do_run()（扫描 src/ 和 specs/ + 生成 sync_log + 写入文件）
  - **TDD**

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/sync.rs:do_run()` — 同步逻辑

  **Acceptance Criteria**:
  - [ ] `SyncCommand` 实现 `DddCommand` trait
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: SyncCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib sync_command --nocapture
    Evidence: .sisyphus/evidence/task-13-sync-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate sync to DddCommand trait`
  - Files: `src/commands/sync.rs`

- [x] 14. 迁移 accept (internal) 命令到 DddCommand trait

  **What to do**:
  - 在 `src/commands/internal.rs` 中定义 `pub struct AcceptCommand;`
  - 实现 `DddCommand` trait:
    - `name()` → `"accept"`
    - `description()` → `"接受阶段计划"`
    - `prompt_template()` → `None`
    - `required_params()` → `vec![]`
    - `execute()` → 封装 accept()（扫描 phases dir + 创建 Phase entries + 保存状态）
    - **关键**: 保留 `let _ =` 错误丢弃行为 — execute 内部 `let _ = accept(ctx); Ok(CommandResult::success("..."))`
  - **TDD**

  **Must NOT do**:
  - 不修复 `let _ =` 错误丢弃行为（pre-existing behavior）
  - accept 仍然是内部命令，在 registry 中标记为 internal

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: T15
  - **Blocked By**: T1, T2

  **References**:
  - `src/commands/internal.rs:accept()` — 当前实现

  **Acceptance Criteria**:
  - [ ] `AcceptCommand` 实现 `DddCommand` trait
  - [ ] execute() 保留错误丢弃行为
  - [ ] `cargo test` 通过

  **QA Scenarios**:

  ```
  Scenario: AcceptCommand trait 实现
    Tool: Bash
    Steps:
      1. cargo test --lib accept_command --nocapture
    Evidence: .sisyphus/evidence/task-14-accept-trait.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate accept to DddCommand trait`
  - Files: `src/commands/internal.rs`

- [x] 15. 重写 mod.rs dispatch 为 registry lookup

  **What to do**:
  - 在 `src/commands/mod.rs` 中:
    - 保留 clap `Cli` struct 和 `Command` enum（仍用于 CLI 解析）
    - `dispatch()` 改为: 从 `CommandRegistry::new()` 获取 registry，根据 enum variant 查找对应 trait 实现，调用 `execute()`
    - 对 `CommandResult` 统一处理:
      - 如果 `result.success` → 打印 `result.message`（如有 prompt 则打印 prompt）
      - 如果 `!result.success` → eprintln! 错误信息
    - 移除各模块的直接 `use` 和 `match` 调用
    - `Setup` 命令特殊处理: 保持 `--tool` 参数解析，传入 registry 引用
  - **关键设计**: clap enum 仍存在用于解析参数，但 dispatch 通过 registry 走 trait object
  - **TDD**: 测试 dispatch 路由正确性

  **Must NOT do**:
  - 不移除 clap（仍然用于 CLI 解析）
  - 不改变 CLI 参数格式（用户接口不变）
  - 不移除 `Command` enum（只是 dispatch 逻辑改变）
  - 保留 `setup.rs` 的 `println!("Error:")` 而非 `eprintln!("错误:")` 风格差异

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []
  - Reason: 这是连接所有命令的中枢，需要仔细处理 dispatch 路由和错误传播

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: T16, FINAL
  - **Blocked By**: T3, T4-T14

  **References**:

  **Pattern References**:
  - `src/commands/mod.rs:dispatch()` — 当前 match 分发逻辑，需要替换
  - `src/commands/mod.rs:Command` — clap enum 定义（保留）
  - `src/commands/mod.rs:Cli` — CLI 入口（保留）
  - `src/commands/registry.rs:CommandRegistry` — 新的查找机制

  **API/Type References**:
  - `src/commands/trait.rs:DddCommand` — trait object 类型
  - `src/commands/trait.rs:CommandResult` — 统一返回类型

  **Acceptance Criteria**:
  - [ ] dispatch() 使用 registry lookup 而非 match 调用各模块
  - [ ] CommandResult 的 message 和 prompt 正确输出
  - [ ] `cargo build` 通过
  - [ ] `cargo test` 通过
  - [ ] `cargo clippy` 无新 warning

  **QA Scenarios**:

  ```
  Scenario: dispatch 路由正确
    Tool: Bash
    Steps:
      1. cargo test --lib dispatch --nocapture
      2. Assert "test result: ok"
    Expected Result: 所有 dispatch 路由测试通过
    Evidence: .sisyphus/evidence/task-15-dispatch-tests.txt

  Scenario: 全量编译和测试
    Tool: Bash
    Steps:
      1. cargo build && cargo test && cargo clippy
      2. Assert all exit code 0
    Evidence: .sisyphus/evidence/task-15-full-build.txt

  Scenario: CLI 帮助信息不变
    Tool: Bash
    Steps:
      1. cargo run -- --help
      2. Assert 输出包含所有子命令名
    Expected Result: "init", "prepare", "exec", "verify", "audit", "confirm", "final", "archive", "report", "sync", "accept", "setup"
    Evidence: .sisyphus/evidence/task-15-help.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): rewrite dispatch to use trait object registry`
  - Files: `src/commands/mod.rs`

- [x] 16. 重写 setup.rs 自动遍历 registry 生成 Skills/Commands

  **What to do**:
  - 在 `src/commands/setup.rs` 中:
    - 移除 `PUBLIC_COMMANDS` 硬编码数组
    - 接收 `&CommandRegistry` 参数
    - 遍历 `registry.all()` 获取所有命令的 name/description/skill_prompt/command_prompt
    - `setup_claude()` 改为遍历 registry 生成 `.claude/commands/ddd-{name}.md`
    - `setup_opencode()` 改为遍历 registry 生成 `.opencode/commands/ddd-{name}.md` + `.opencode/skills/ddd-{name}.md`
    - `prepare_init_file()` 保持不变（不涉及命令遍历）
  - `setup` 本身也实现 `DddCommand` trait（但它的 execute 是特殊的，需要额外参数 --tool）
  - **TDD**: 测试 setup 生成的文件内容正确性

  **Must NOT do**:
  - 不改变生成的文件格式（Claude 的 PromptTask JSON、Opencode 的 Markdown 格式）
  - 不改变 `prepare_init_file()` 逻辑
  - 保留 `println!("Error:")` 的风格（不统一为 eprintln）
  - setup 命令本身不注册到 PUBLIC_COMMANDS 生成列表中（它不是工作流命令）

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: FINAL
  - **Blocked By**: T15

  **References**:

  **Pattern References**:
  - `src/commands/setup.rs:PUBLIC_COMMANDS` — 要移除的硬编码列表
  - `src/commands/setup.rs:make_prompt()` — 要迁移到各命令的 command_prompt() 方法
  - `src/commands/setup.rs:setup_claude()` — 改为遍历 registry
  - `src/commands/setup.rs:setup_opencode()` — 改为遍历 registry
  - `src/commands/setup.rs:PromptTask` — Claude JSON 结构（保留）

  **API/Type References**:
  - `src/commands/registry.rs:CommandRegistry` — all() 方法
  - `src/commands/trait.rs:DddCommand` — skill_prompt(), command_prompt()

  **Acceptance Criteria**:
  - [ ] `PUBLIC_COMMANDS` 硬编码已移除
  - [ ] setup 从 registry 自动遍历生成文件
  - [ ] 生成的文件内容与重构前一致
  - [ ] `cargo test` 通过
  - [ ] `cargo clippy` 无新 warning

  **QA Scenarios**:

  ```
  Scenario: setup --tool claude 生成正确
    Tool: Bash
    Steps:
      1. cargo run -- setup --tool claude
      2. ls .claude/commands/ | wc -l
      3. Assert 11 files generated (ddd-init.md through ddd-sync.md)
    Expected Result: 11 个 PromptTask JSON 文件，内容正确
    Evidence: .sisyphus/evidence/task-16-setup-claude.txt

  Scenario: setup --tool opencode 生成正确
    Tool: Bash
    Steps:
      1. cargo run -- setup --tool opencode
      2. ls .opencode/commands/ | wc -l
      3. ls .opencode/skills/ | wc -l
      4. Assert 11 commands + 11 skills generated
    Expected Result: 11 个 command md + 11 个 skill md 文件
    Evidence: .sisyphus/evidence/task-16-setup-opencode.txt

  Scenario: 生成的文件内容与重构前一致
    Tool: Bash
    Steps:
      1. git stash (保存重构后的代码)
      2. 切到重构前的 commit，运行 setup
      3. 比较生成文件差异
      4. git stash pop
    Expected Result: 无差异或仅有预期内的差异
    Evidence: .sisyphus/evidence/task-16-diff.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): setup auto-traverses registry for generation`
  - Files: `src/commands/setup.rs`

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo build` + `cargo clippy` + `cargo test`. Review all changed files for: `unwrap()` in production code, empty catches, dead code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Run every sub-command: `ddd init`, `ddd prepare`, `ddd exec`, `ddd verify`, `ddd audit`, `ddd confirm`, `ddd final`, `ddd archive`, `ddd report`, `ddd sync`, `ddd accept`. Verify each produces expected output. Run `ddd setup --tool claude` and `ddd setup --tool opencode` — verify generated files match expectations. Save evidence to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **T1**: `refactor(commands): create lib.rs for integration test support` - src/lib.rs
- **T2**: `refactor(commands): define DddCommand trait, CommandResult, and CommandRegistry` - src/commands/trait.rs, src/commands/registry.rs
- **T3**: `fix(prompts): enhance render validation and fix anem typo` - src/prompts/mod.rs
- **T4-T14**: `refactor(commands): migrate {name} to DddCommand trait` - src/commands/{name}.rs
- **T15**: `refactor(commands): rewrite dispatch to use trait object registry` - src/commands/mod.rs
- **T16**: `refactor(commands): setup auto-traverses registry for generation` - src/commands/setup.rs

---

## Success Criteria

### Verification Commands
```bash
cargo build          # Expected: Compiles successfully
cargo test           # Expected: All tests pass (0 failures)
cargo clippy         # Expected: No new warnings vs baseline
ddd init --help      # Expected: Shows init usage
ddd setup --help     # Expected: Shows setup usage
ddd setup --tool claude  # Expected: Generates .claude/commands/ files
ddd setup --tool opencode # Expected: Generates .opencode/commands/ + skills/ files
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] CLI behavior identical to pre-refactoring
- [ ] setup generates identical output to pre-refactoring
