# Fix setup.rs: Command/Skill 生成同步新模式

## TL;DR

> **Quick Summary**: 修复 setup.rs 使其使用 command_prompt()/skill_prompt() 返回值生成文件，而非硬编码模板。修复文件名双重前缀 bug。为 setup_claude() 增加 skills 文件生成。
>
> **Deliverables**:
> - 修复文件名双重 ddd- 前缀 bug
> - setup_opencode() command 文件使用 command_prompt() 返回值（保留 frontmatter）
> - setup_claude() 增加 .claude/skills/ 文件生成循环（纯 Markdown）
>
> **Estimated Effort**: Medium（单文件多逻辑变更）
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Task 1 (prefix) → Tasks 2,3 (parallel) → Task 4 (tests + verify)

---

## Context

### Original Request
用户调查发现 setup 命令没有按照新的 Skill 加载模式生成 command 和 skill 文件。要求修复并增加 Claude skills 生成。

### Interview Summary
**Key Decisions**:
- 文件名前缀：去掉 format 中的 `ddd-`，直接用 `name()` 返回值
- Command frontmatter：保留 YAML frontmatter + command_prompt() 作为 body
- Claude skills 格式：纯 Markdown（不用 PromptTask JSON 封装）

### Metis Review
**Critical Findings**:
- 🚨 文件名双重前缀 bug：`format!("ddd-{}.md", cmd.name())` → `ddd-ddd-init.md`
- 🚨 command_prompt() 返回 Skill 加载文本，不是 Bash 命令 — 语义转变已确认
- 🚨 SyncCommand.name() 返回 "sync" 无前缀 — strip_prefix 需兼容

---

## Work Objectives

### Core Objective
setup_opencode() 和 setup_claude() 的文件生成逻辑与 command_prompt()/skill_prompt() 返回值同步。

### Concrete Deliverables
- `src/commands/setup.rs` — 4 处修改
- 文件名从 `ddd-{ddd-init}` → `{ddd-init}`

### Definition of Done
- [ ] `cargo build` 零错误
- [ ] `cargo test` 全通过
- [ ] 手动验证生成的文件内容正确

### Must Have
- 文件名正确（无双重 ddd- 前缀）
- command 文件保留 YAML frontmatter，body 使用 command_prompt()
- skill 文件使用 skill_prompt() 纯 Markdown
- setup_claude() 生成 .claude/skills/ 目录和文件

### Must NOT Have (Guardrails)
- 不修改 DddCommand trait、PromptTask struct、任何 command impl 文件
- 不修改 setup_opencode() 中已有的 skill 文件生成逻辑（L127-137）
- 不重构 setup_claude() 和 setup_opencode() 共享逻辑
- 不修改 SyncCommand.name() — 让 strip_prefix 兼容无前缀的情况

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (cargo test)
- **Automated tests**: Tests-after（现有测试 + 手动验证）
- **Framework**: cargo test

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (foundation - file name fix):
└── Task 1: 修复文件名双重 ddd- 前缀 bug [quick]

Wave 2 (parallel - two independent fixes):
├── Task 2: setup_opencode() command 文件使用 command_prompt() [quick]
└── Task 3: setup_claude() 增加 skills 文件生成循环 [quick]

Wave 3 (verification):
└── Task 4: 编译验证 + 手动测试 [quick]
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 | - | 2, 3 |
| 2 | 1 | 4 |
| 3 | 1 | 4 |
| 4 | 2, 3 | - |

---

## TODOs

- [x] 1. 修复文件名双重 ddd- 前缀 bug

  **What to do**:
  - 在 `src/commands/setup.rs` 中找到所有 `format!("ddd-{}.md", ...)` 的文件名生成
  - 改为直接使用 `cmd.name()` 返回值作为文件名（去掉硬编码的 `ddd-` 前缀）
  - 例如：`format!("ddd-{}.md", cmd.name())` → `format!("{}.md", cmd.name())`
  - 涉及位置：
    - setup_opencode() 的 command 文件名（约 L116）
    - setup_opencode() 的 skill 文件名（约 L132）
    - setup_claude() 的 command 文件名（约 L73）

  **Must NOT do**:
  - 不修改 cmd.name() 的返回值
  - 不修改 SyncCommand 的实现

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 2, 3
  - **Blocked By**: None

  **References**:
  - `src/commands/setup.rs:73,116,132` — 三个文件名生成位置
  - `src/commands/sync.rs` — SyncCommand.name() 返回 "sync"（无 ddd- 前缀），strip 后仍为 "sync"

  **Acceptance Criteria**:
  - [ ] 所有文件名使用 `format!("{}.md", cmd.name())`
  - [ ] SyncCommand 生成 "sync.md" 而非 "ddd-sync.md"

  **QA Scenarios**:
  ```
  Scenario: 文件名无双重前缀
    Tool: Bash (grep)
    Steps:
      1. grep -n 'ddd-.*\.md' src/commands/setup.rs
      2. 确认不存在 format!("ddd-{}.md" 模式
    Expected Result: 无 "ddd-{}" 格式，全部使用 "{}.md"
    Evidence: .sisyphus/evidence/task-1-filenames.txt
  ```

  **Commit**: NO (group with final)

- [x] 2. setup_opencode() command 文件使用 command_prompt()

  **What to do**:
  - 在 `src/commands/setup.rs` 的 `setup_opencode()` 中
  - 找到 command 文件生成的硬编码模板（约 L116-125）：
    ```rust
    let command_content = format!(
        r#"---
    description: {description}
    agent: Sisyphus
    ---

    !`{binary} {name} $ARGUMENTS 2>&1`
    "#
    );
    ```
  - 替换为：保留 YAML frontmatter，body 改用 registry 中已存的 `command_prompt` 值：
    ```rust
    let command_content = format!(
        r#"---
    description: {description}
    agent: Sisyphus
    ---

    {command_prompt}
    "#
    );
    ```
  - 其中 `command_prompt` 来自 `cmd_data.command_prompt`（已在 L72 收集）

  **Must NOT do**:
  - 不修改 YAML frontmatter 格式
  - 不修改 setup_opencode() 的 skill 文件生成逻辑（L127-137）
  - 不删除或修改 JSON registry 数据收集（L65-79）

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 3)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 4
  - **Blocked By**: Task 1

  **References**:
  - `src/commands/setup.rs:65-79` — JSON registry 数据收集，command_prompt 值已存入 cmd_data
  - `src/commands/setup.rs:111-125` — 当前硬编码 command 文件生成
  - `src/commands/setup.rs:78-91` (setup_claude) — 参考如何使用 command_prompt 值

  **Acceptance Criteria**:
  - [ ] command 文件 body 使用 registry 中的 command_prompt 值
  - [ ] YAML frontmatter（description + agent: Sisyphus）保留不变

  **QA Scenarios**:
  ```
  Scenario: command 文件包含 frontmatter + prompt body
    Tool: Bash (grep)
    Steps:
      1. grep -n "command_prompt\|!`" src/commands/setup.rs
      2. 确认 command 文件生成模板中无 !`backtick` 语法
      3. 确认使用了 cmd_data.command_prompt 或等效变量
    Expected Result: 无硬编码 !`{binary}，使用 command_prompt 变量
    Evidence: .sisyphus/evidence/task-2-command-prompt.txt
  ```

  **Commit**: NO (group with final)

- [x] 3. setup_claude() 增加 skills 文件生成循环

  **What to do**:
  - 在 `src/commands/setup.rs` 的 `setup_claude()` 函数中
  - 在现有的 command 文件生成循环（L78-91）之后，增加一个新的 skills 文件生成循环
  - 新循环逻辑：
    1. 创建 `.claude/skills/` 目录：`fs::create_dir_all(claude_dir.join("skills"))`
    2. 遍历 registry 中的命令
    3. 对每个命令，调用 `cmd.skill_prompt(binary, cmd.name())`
    4. 如果返回 `Some(content)`，写入 `{claude_dir}/skills/{cmd.name()}.md`
    5. 如果返回 `None`，跳过（不创建空文件）
  - 文件格式：纯 Markdown（不使用 PromptTask JSON 封装）
  - 参考模板：
    ```rust
    // 在 command 文件生成循环之后
    let skills_dir = claude_dir.join("skills");
    fs::create_dir_all(&skills_dir)?;

    for cmd in registry.all_commands() {
        if let Some(content) = cmd.skill_prompt(ddd_binary.to_string_lossy().as_ref(), cmd.name()) {
            let skill_path = skills_dir.join(format!("{}.md", cmd.name()));
            fs::write(&skill_path, &content)?;
        }
    }
    ```

  **Must NOT do**:
  - 不修改已有的 command 文件生成逻辑
  - 不用 PromptTask JSON 封装 skill 文件
  - 不修改 setup_opencode() 的 skill 生成

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 4
  - **Blocked By**: Task 1

  **References**:
  - `src/commands/setup.rs:60-97` — setup_claude() 完整函数
  - `src/commands/setup.rs:78-91` — 现有 command 文件生成循环（作为 skills 循环的参考结构）
  - `src/commands/setup.rs:127-137` (setup_opencode) — 另一个 skill 文件生成参考

  **Acceptance Criteria**:
  - [ ] setup_claude() 创建 .claude/skills/ 目录
  - [ ] 遍历 registry 生成 skill 文件
  - [ ] 使用 skill_prompt() 返回值作为纯 Markdown 内容
  - [ ] None 返回值跳过文件创建

  **QA Scenarios**:
  ```
  Scenario: skills 目录和文件生成
    Tool: Bash (grep)
    Steps:
      1. grep -n "skills_dir\|skill_prompt" src/commands/setup.rs
      2. 确认 setup_claude() 中有 skills_dir 创建和 skill_prompt 调用
    Expected Result: setup_claude() 中存在 skills 循环
    Evidence: .sisyphus/evidence/task-3-claude-skills.txt
  ```

  **Commit**: NO (group with final)

- [x] 4. 编译验证 + 手动测试

  **What to do**:
  - 运行 `cargo build 2>&1` 确认零错误
  - 运行 `cargo test 2>&1` 确认全通过
  - 运行 `cargo clippy 2>&1` 确认无新增 warning

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Tasks 2, 3

  **References**: 无

  **Acceptance Criteria**:
  - [ ] cargo build 零错误
  - [ ] cargo test 全通过
  - [ ] cargo clippy 无新增 warning

  **QA Scenarios**:
  ```
  Scenario: 编译通过
    Tool: Bash
    Steps:
      1. cargo build 2>&1
      2. cargo test 2>&1
    Expected Result: 0 errors, all tests pass
    Evidence: .sisyphus/evidence/task-4-build-test.txt
  ```

  **Commit**: YES
  - Message: `fix(setup): sync file generation with new Skill loading mode`
  - Files: `src/commands/setup.rs`
  - Pre-commit: `cargo build && cargo test`

---

## Final Verification Wave

- [x] F1. **Build Verification** — cargo build + cargo test + cargo clippy
- [x] F2. **Diff Audit** — git diff --stat 确认仅修改 setup.rs

## Commit Strategy

- **Single Commit**: `fix(setup): sync file generation with new Skill loading mode`
  - File: `src/commands/setup.rs`
  - Pre-commit: `cargo build && cargo test`

## Success Criteria

### Verification Commands
```bash
cargo build 2>&1   # Expected: zero errors
cargo test 2>&1    # Expected: all tests pass
git diff --stat    # Expected: 1 file (setup.rs)
```

### Final Checklist
- [ ] 文件名无双重 ddd- 前缀
- [ ] command 文件使用 command_prompt() body + 保留 frontmatter
- [ ] setup_claude() 生成 .claude/skills/ 文件
- [ ] 编译和测试通过
