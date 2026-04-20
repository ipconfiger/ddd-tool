# DDD - 文档驱动开发框架

## 开篇

你用过 AI 编程 Agent 吗？Claude Code、OpenCode、Cline……

刚上手时觉得真香，让它写个功能，几分钟就出来了。但多轮对话之后，问题来了：

**上下文越来越长，AI 开始"失忆"了。** 它不再记得最初的需求目标，做出来的东西越来越偏离轨道。更要命的是——**Token 账单也在疯狂攀升**，几轮对话下来，消耗的 tokens 比代码本身还贵。

如果你同时跑多个 Agent 并发开发……情况更糟：各自拿着不同的上下文，做出来的东西风格割裂，甚至功能重叠或冲突。

**DDD 解决的就是这个问题。**

它不是又一个 AI 编程工具，而是一套**上下文约束协议**：每个阶段用文档锚定目标，用状态机控制开发节奏。AI Agent 只需知道"我现在该做 Phase1"和"Phase1 的 spec 在哪里"，剩下的——它不会忘，你也不必担心 Token 爆炸。

## 设计理念

DDD 的核心设计围绕三个问题：

**1. AI 忘了需求怎么办？**
用 `roadmap.json` 把项目拆成若干阶段（Phase），每个阶段配套一份 spec 文档。AI 进入任何阶段，都能立刻知道"这个阶段要做什么"。

**2. 多 Agent 并发 Token 爆炸怎么办？**
每个 Agent 只加载当前阶段的上下文，而不是整本需求文档。阶段间通过状态机推进，避免重复加载历史对话。

**3. 做到哪了？谁来把关？**
每个阶段有明确的状态：`init` → `dev` → `verify` → `fixing` → `finished`。不靠人盯着文档回忆，靠状态机自动推进。

**核心原则：文档先行，状态驱动，Token 高效。**

## 安装

**方式一：直接安装（推荐）**

```bash
cargo install ddd
```

**方式二：从源码安装**

```bash
git clone https://github.com/ipconfiger/ddd-tool.git
cd ddd-tool
cargo install --path .
```

安装完成后运行 `ddd --help` 验证。

## 快速开始

**第一步：部署 DDD 命令到项目**

在项目根目录运行：

```bash
ddd setup --tool opencode
```

这会将 DDD 的 slash 命令（`/ddd-init`、`/ddd-exec`、`/ddd-verify` 等）部署到项目中。

**第二步：在 OpenCode 中初始化项目**

```
/ddd-init ./project_docs/需求文档.md
```

这会在 `project_docs/` 下生成 `roadmap.json`，包含项目的阶段划分。

**第三步：准备阶段文档**

```
/ddd-prepare
```

根据 roadmap.json 中的 phases，生成对应的 spec 文档框架。

**第四步：启动开发**

```
/ddd-exec
```

DDD 会读取当前 Phase 的 spec 文档，渲染开发指令，启动 AI Agent 开始工作。

**第五步：审核成果**

```
/ddd-verify
```

进入审核模式，检查当前阶段是否符合 spec 要求。

**典型开发循环**

```
/ddd-exec   # 开发当前 Phase
/ddd-verify # 审核
/ddd-exec   # 继续下一 Phase
...
```

## 核心命令一览

| 命令 | 说明 |
|------|------|
| `/ddd-init <文档>` | 初始化项目，生成 roadmap.json |
| `/ddd-prepare` | 准备阶段文档（spec） |
| `/ddd-exec` | 执行当前阶段的开发 |
| `/ddd-verify` | 审核当前阶段成果 |
| `/ddd-fix-plan` | 制定修复计划 |
| `/ddd-fix-exec` | 执行修复 |
| `/ddd-report` | 生成阶段报告 |
| `/ddd-archive` | 归档已完成的项目 |

## 工作流状态机

```
┌─────────┐
│  init   │  ← 阶段初始化
└────┬────┘
     │ /ddd-exec
     ▼
┌─────────┐
│   dev   │  ← AI Agent 开发中
└────┬────┘
     │ /ddd-verify
     ▼
┌──────────────┐
│ issue_found  │  ← 发现问题
└──────┬───────┘
       │ /ddd-fix-plan && /ddd-fix-exec
       ▼
┌─────────┐
│ fixing  │  ← 修复中
└────┬────┘
     │ /ddd-finish-fix
     ▼
┌─────────┐     │ /ddd-verify
│   dev   │─────┘  ← 重新审核
└────┬────┘
     │ /ddd-finish-phrase
     ▼
┌──────────┐
│ finished │  ← 阶段完成，自动推进下一阶段
└──────────┘
```

## License

MIT
