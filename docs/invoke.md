# OpenCode Skill 与 Command 互调机制调研

## 调研目标

研究如何通过 Command 明确调用 Skill，在 Skill 中明确调用其他 Skill，实现多任务串联执行，并在 Skill 中调用 Bash 命令更新状态。

---

## 核心架构

```
用户输入 /command
       │
       ▼
  Command (.opencode/commands/*.md)
       │  ┌─ frontmatter: description, agent, model, subtask
       │  └─ body: 模板内容（支持变量替换、Bash 注入、文件引用）
       ▼
  指定 Agent 执行（agent frontmatter 决定）
       │
       ▼
  Agent 通过 skill() 工具加载 Skill
       │
       ▼
  Skill (.opencode/skills/*.md) 提供行为指令
       │  可在指令中要求 Agent 加载其他 Skill
       │  可通过 Agent 执行 Bash 命令更新状态
       ▼
  任务完成
```

---

## 三大核心机制

### 1. Bash 输出注入 — `!`反引号`` 语法

**这是串联执行的关键**：在 Command 或 Skill 的模板内容中，使用 `!`bash命令`` 语法，会在发送给 Agent **之前**执行 Bash 命令，并将 stdout 注入到 prompt 中。

```markdown
---
description: 初始化 DDD 项目
agent: Sisyphus
---

!`ddd-tool init $ARGUMENTS 2>&1`
```

**工作原理**：
1. 用户执行 `/ddd-init my-project`
2. OpenCode 先执行 `ddd-tool init my-project 2>&1`
3. 将命令输出（如 JSON 状态、文件列表等）作为 prompt 发送给 Sisyphus Agent
4. Agent 根据输出内容执行后续操作

**关键特性**：
- Bash 命令在 prompt 注入**之前**执行，不是由 Agent 执行
- 输出直接成为 prompt 内容的一部分
- 可以读取文件内容、查询状态、生成上下文
- 错误输出通过 `2>&1` 也会被捕获

### 2. Command → Agent → Skill 调用链

Command 通过 `agent` frontmatter 指定执行者，Agent 在 prompt 中被指示加载特定 Skill。

**Command 层**（`.opencode/commands/review.md`）：

```markdown
---
description: Run code review on files or recent changes
---

对以下内容进行 Code Review：$ARGUMENTS
加载 code-review skill，根据文件类型决定是否加载其他 skill。
```

**Agent 层**（`.opencode/agent/reviewer.md`）：

```markdown
---
mode: subagent
model: opus
---

## Prime Directive

1. 使用 skill 工具加载 code-review skill
2. 如果是前端代码，同时加载 frontend-philosophy skill
3. 如果是后端代码，同时加载 code-philosophy skill
4. 如果是 Plan 审查，加载 plan-review + code-philosophy skill
```

**调用流程**：
```
/review src/auth.ts
  → review command 加载，模板中 $ARGUMENTS = "src/auth.ts"
  → 分配给 reviewer Agent
  → reviewer Agent 的 system prompt 指示它加载 skill("code-review")
  → reviewer Agent 判断是后端代码，额外加载 skill("code-philosophy")
  → 两个 Skill 的指令合并，指导 Agent 执行审查
```

### 3. Skill 内调用其他 Skill（通过 Agent 指令）

**Skill 本身不能直接调用其他 Skill**。Skill 是纯 Markdown 指令文档。但 Skill 的指令中可以**指示 Agent 加载其他 Skill**：

**方案 A：Skill 指令中明确要求加载**

```markdown
---
name: ddd-exec
description: 执行 DDD 任务
---

## 执行流程

1. 先加载 ddd-prepare skill 进行任务准备
2. 执行核心逻辑
3. 加载 ddd-verify skill 验证结果

!`ddd-tool exec $ARGUMENTS 2>&1`
```

> ⚠️ 注意：Skill 中写 `加载 ddd-prepare skill` 是**给 Agent 看的自然语言指令**，Agent 会用 `skill({name: 'ddd-prepare'})` 工具去执行。Skill 本身没有"调用"能力，它通过 Agent 间接实现。

**方案 B：通过 Command 编排多个 Skill**

```markdown
---
description: 完整 DDD 工作流
agent: Sisyphus
---

## 任务

按以下顺序执行：

1. 加载 ddd-init skill，完成初始化
2. 加载 ddd-prepare skill，准备数据
3. 加载 ddd-exec skill，执行核心任务
4. 加载 ddd-verify skill，验证结果
5. 加载 ddd-report skill，生成报告

项目路径：!`pwd`
当前状态：!`cat .ddd/state.json 2>/dev/null || echo "无状态文件"`
```

---

## 模板语法参考

### Command 模板变量

| 语法 | 说明 | 示例 |
|------|------|------|
| `$ARGUMENTS` | 用户传入的完整参数 | `/cmd foo bar` → `foo bar` |
| `$1` `$2` `$3` | 位置参数 | `/cmd a b c` → `$1=a`, `$2=b`, `$3=c` |
| `` !`cmd` `` | Bash 输出注入 | `` !`date` `` → `2026-06-06` |
| `@filename` | 文件内容注入 | `@AGENTS.md` → 文件全文 |

### Skill Frontmatter

```yaml
---
name: my-skill           # 必填，小写字母+数字+连字符，必须与目录名一致
description: 技能描述     # 1-1024 字符，用于 <available_skills> 展示和触发匹配
---
```

### Command Frontmatter

```yaml
---
description: 命令描述      # 用于命令列表展示
agent: Sisyphus           # 指定执行的 Agent
model: opus               # 可选，覆盖 Agent 默认模型
subtask: true             # 可选，强制以 subagent 模式执行
---
```

---

## 实战模式：多任务串联

### 模式 1：Command 驱动的流水线

通过一个 Command 编排整个流程，利用 `!`反引号`` 在每个阶段读取状态：

```markdown
---
description: DDD 全流程
agent: Sisyphus
---

## 全流程执行

当前项目状态：
!`ddd-tool status 2>&1`

待执行任务：
!`ddd-tool list-tasks --pending 2>&1`

按照以下流程执行，每完成一步，用 Bash 更新状态：

### Step 1: 准备
- 加载 ddd-prepare skill
- 执行完毕后运行: `ddd-tool update-status prepared`

### Step 2: 执行
- 加载 ddd-exec skill
- 执行完毕后运行: `ddd-tool update-status executed`

### Step 3: 验证
- 加载 ddd-verify skill
- 执行完毕后运行: `ddd-tool update-status verified`

### Step 4: 报告
- 加载 ddd-report skill
- 执行完毕后运行: `ddd-tool update-status reported`
```

### 模式 2：Skill 链式调用

在 Skill 指令中要求 Agent 在适当时机加载下一个 Skill：

```markdown
---
name: ddd-verify
description: 验证 DDD 任务结果
---

## 验证流程

验证规则：!`ddd-tool get-rules verify 2>&1`

执行验证步骤...（此处为具体验证指令）

### 验证通过后

如果验证全部通过：
1. 用 Bash 执行 `ddd-tool update-status verified` 更新状态
2. 加载 ddd-report skill 生成报告
3. 在 ddd-report skill 执行完毕后，加载 ddd-final skill 完成收尾

### 验证失败

如果验证发现问题：
1. 用 Bash 执行 `ddd-tool update-status failed --reason "..."` 记录失败
2. 加载 ddd-exec skill 重新执行（最多重试 3 次）
3. 如果仍然失败，停止并向用户报告
```

### 模式 3：状态机驱动的 Command

利用 `!`反引号`` 读取当前状态，动态决定下一步：

```markdown
---
description: DDD 继续执行
agent: Sisyphus
---

当前阶段：!`ddd-tool get-current-phase 2>&1`
项目状态：!`cat .ddd/state.json 2>&1`

根据当前阶段，加载对应的 Skill 继续执行：
- 如果阶段是 `prepared`，加载 ddd-exec skill
- 如果阶段是 `executed`，加载 ddd-verify skill
- 如果阶段是 `verified`，加载 ddd-report skill
- 如果阶段是 `reported`，加载 ddd-final skill
- 如果阶段是 `init`，加载 ddd-prepare skill

每步执行完后，用 Bash 命令 `ddd-tool advance-phase` 推进状态。
```

---

## 关键限制与注意事项

### Skill 不是可执行程序

```
❌ 错误理解：Skill 可以直接调用其他 Skill 或 Bash
✅ 正确理解：Skill 是给 Agent 看的行为指令，Agent 是实际执行者
```

Skill 中写 "加载 xxx skill" 或 "执行 bash 命令"，都是给 Agent 的**自然语言指令**。Agent 读取后决定是否用 `skill()` 工具或 `bash` 工具去执行。

### `!`反引号`` 的执行时机

```
❌ 错误理解：!`cmd` 是 Agent 执行的 Bash 命令
✅ 正确理解：!`cmd` 在模板渲染阶段执行，输出注入到 prompt 中，Agent 看到的是执行结果
```

这意味着：
- `!`cmd`` 适合读取状态、获取上下文
- 不适合需要 Agent 判断后再执行的操作
- 需要 Agent 动态执行 Bash 时，应在 Skill 指令中用自然语言要求

### Agent 权限控制

Agent 能否执行 Bash、加载 Skill，取决于其 `permission` 配置：

```jsonc
// opencode.json
{
  "agent": {
    "my-agent": {
      "permission": {
        "bash": { "allow": ["ddd-tool *", "git *"] },  // 只允许特定命令
        "skill": { "allow": ["ddd-*"] }                 // 只允许特定 Skill
      }
    }
  }
}
```

---

## 本项目中的实际应用

本项目已有 11 个 DDD Command 和对应的 11 个 DDD Skill：

| Command | 对应 Skill | 职责 |
|---------|-----------|------|
| `/ddd-init` | `ddd-init` | 初始化项目 |
| `/ddd-prepare` | `ddd-prepare` | 准备数据 |
| `/ddd-exec` | `ddd-exec` | 执行核心任务 |
| `/ddd-verify` | `ddd-verify` | 验证结果 |
| `/ddd-confirm` | `ddd-confirm` | 确认完成 |
| `/ddd-report` | `ddd-report` | 生成报告 |
| `/ddd-final` | `ddd-final` | 最终收尾 |
| `/ddd-sync` | `ddd-sync` | 同步状态 |
| `/ddd-audit` | `ddd-audit` | 审计检查 |
| `/ddd-accept` | `ddd-accept` | 接受结果 |
| `/ddd-archive` | `ddd-archive` | 归档 |

所有 Command 统一由 **Sisyphus Agent** 执行，通过 `!`ddd-tool <subcommand> $ARGUMENTS 2>&1`` 注入 Bash 输出。

### 改进方向：串联执行

当前设计是每个 Command 独立执行。若需串联，可：

1. **新增编排 Command**：创建 `/ddd-pipeline` Command，按顺序调用各阶段 Skill
2. **Skill 内指示链式调用**：在 `ddd-verify` 中指示验证通过后加载 `ddd-report`
3. **状态机 Command**：`/ddd-continue` 根据当前状态自动加载下一个 Skill

---

## 总结

| 机制 | 如何实现 | 适用场景 |
|------|---------|---------|
| Command 调用 Skill | Command 的 Agent prompt 中指示 `skill({name: 'xxx'})` | 固定流程入口 |
| Skill 调用 Skill | Skill 指令中要求 Agent 加载另一个 Skill | 条件性链式执行 |
| Bash 状态注入 | Command/Skill 中 `!`cmd`` | 读取当前状态、上下文 |
| Agent 执行 Bash | Skill 指令中要求 Agent 运行 Bash 命令 | 动态更新状态 |
| 状态驱动的流程 | `!`cmd`` 读取状态 + 条件性加载 Skill | 自适应流水线 |

**核心公式**：

```
串联执行 = !`backtick`（获取状态）+ Skill 指令（指导行为）+ Agent（实际执行）
```
