# DDD - 文档驱动开发框架
**注意** 本项目停止更新，所有功能都在新项目中： [[https://github.com/ipconfiger/ForceLoop]](https://github.com/ipconfiger/ForceLoop)


## 开篇: AI Coding 的工程化难题

当前最顶的几档大模型, 无论是国外的还是国内的, 开源的还是闭源的, 其编程的能力都是很强的. 但是为什么在实际用Coding Agent, 无论是Claude Code, CodeX, 还是OpenCode, 在开发大型项目的时候, 都会遇到各式各样的问题, 不是容易遗忘丢失记忆,从而开始胡编乱造, 还是满口说着好好好, 打开一看全是Mock, Stub, 真正干活的代码没写几行.

归结其原因, 大体就是上下文窗口尺寸的限制, 因为如果说LLM是用了已知网络的知识来构建的大脑, 上下文窗口其实就是Agent的短期记忆, 而大部分的LLM的上下文窗口只有200K, 多的有1M. 看起来能装几本小说的内容, 看着很多, 但是在实际使用的时候, 会被thinking, 大篇大篇的输出(有的模型很啰嗦), 调用shell返回的大量的垃圾日志, 读取的代码文件占据. 如果不加节制, 很快就满了.

所以如何让Coding Agent能够几行需求就一次性生成整个程序, 落实到工程上, 就是如何合理的利用上下文窗口, 让Agent能够拿到尽量精简干净的上下文, 在200K窗口内, 严格按照规格生成制成品, 如果要做的程序很大, 那么就需要切分成多个小的开发阶段, 一个阶段一个阶段的实现, 每一个阶段都能在一个上下文窗口中完成, 由现在大模型的能力,完全能够保证质量, 最后每一个阶段完成后, 有一个总体蓝图能够完成组装, 最后完成整个程序的开发.

[![使用演示](https://i.imgur.com/F6OPZXH.png)](https://youtu.be/4PFDykhqXCI)

## 设计理念

DDD 的核心设计围绕三个问题：

**1. 在有限的上下文窗口中保证不丢失上下文**
编码需要的上下文, 统统用切分好的规格和任务文档来存储, 每次编码只加载对应的规格和任务文档, 任务和主Agent隔离, 完全不依赖主Agent的上下文, 整个过程可以随时清理掉主Agent的上下文窗口,而不会影响构建的作业.

**2. 多 Agent 并发触发Coding Plan超限怎么办？**
对驱动多Agent开发保持克制, 仅在规格设计阶段会并发, 进入开发阶段后,大部分都是串行启动子任务.

**3. 自动分阶段, 自动回归自审核？**
明确的分成文档准备和开发循环两个阶段, 文档准备分为规格设计和任务制定两个阶段, 每个阶段都有自审核,并且文档准备阶段的文档都是人类可读, 可编辑, 可以随时人工介入修改.

开发阶段由任务的状态机来推进, 按照 exec -> verify -> confirm 的循环, 每一步都有验证审核, 保证最终成品严格符合设计.

**核心原则：文档为根，状态驱动。**

## 安装

**方式一：直接安装（推荐）**

```bash
cargo install ddd-tool
```

**方式二：从源码安装**

```bash
git clone https://github.com/ipconfiger/ddd-tool.git
cd ddd-tool
cargo install --path .
```

安装完成后运行 `ddd-tool --help` 验证。

## 快速开始

**第一步：部署 DDD 命令到项目**

在项目根目录运行：

```bash
ddd setup --tool opencode
```

或为 Claude Code 部署：

```bash
ddd setup --tool claude
```

这会将 DDD 的 slash 命令（`/ddd-init`、`/ddd-exec`、`/ddd-verify` 等）部署到项目中。

**第二步：初始化项目**

```
/ddd-init ./project_docs/需求文档.md
```

DDD 会输出一个 Prompt，指导 AI Agent 将需求文档拆分为规格文档（specs），存储到 `project_docs/specs/` 目录下。

**第三步：生成开发计划**

```
/ddd-prepare
```

DDD 会输出一个 Prompt，指导 AI Agent 根据规格文档生成分阶段开发计划，存储到 `project_docs/phases/` 目录下。

在这个阶段, 你可以人工修改, 为了保证修改后文档的一致性, 可以用命令
```
/ddd-audit
```
来对规格和任务进行交叉评审, 消除歧义

**第四步：批准开发计划**

```
/ddd-accept
```

扫描 `project_docs/phases/` 目录，生成 phases 数组，更新 `roadmap.json`。

**第五步：启动开发**

```
/ddd-exec
```

DDD 会读取当前 Phase 的开发计划文档，渲染开发指令，启动 AI Agent 开始工作。

开发完成后, 可以通过下面的命令启动高精度代码审核
```
/ddd-verify
```
进入审核模式，检查当前阶段是否符合 spec 要求。如有问题进入修复流程。

当确认问题均已修复后, 执行命令
```
/ddd-confirm
```
确认当前波次任务开发完成, 进入下一个阶段

**典型开发循环**

```
/ddd-exec      # 开发当前 Phase
/ddd-verify    # 审核
/ddd-confirm   # 确认结果, 推进到下一个波次的开发
...
```

**所有阶段完成后**

```
/ddd-archive
```

归档项目文档到 `project_docs/archives/` 目录，重置 roadmap.json。

## 核心命令一览

| 命令                                    | 说明                                         |
|---------------------------------------|--------------------------------------------|
| `ddd init <文档>`                       | 初始化项目，生成规格文档（specs）                        |
| `ddd prepare`                         | 生成分阶段开发计划（phases）                          |
| `ddd audit`                           | 对开发计划（phases）和规则进行交叉评审                     |
| `ddd accept`                          | 扫描 phases 目录，生成 phases 数组，更新 roadmap.json |
| `ddd exec`                            | 执行当前阶段的开发                                  |
| `ddd verify`                          | 验证当前阶段是否符合 spec 要求                         |
| `ddd confirm`                         | 确认当前阶段开发完成, 进入下一个阶段                        |
| `ddd sync`                            | 反向同步代码到文档                                  |
| `ddd report`                          | 生成项目状态报告                                   |
| `ddd archive`                         | 归档已完成的项目到 archives 目录                      |
| `ddd setup --tool <claude\|opencode>` | 在项目级别配置命令和技能                               |

## 工作流状态机

**全局工作流状态（workflow）**

```
┌─────────────┐
│ initialized │  ← 项目初始化
└──────┬──────┘
       │ ddd init
       ▼
┌─────────────┐
│  doc_ready  │  ← 规格文档已生成
└──────┬──────┘
       │ ddd prepare
       ▼
┌─────────────┐
│  planing    │  ← 开发计划已生成
└──────┬──────┘
       │ ddd accept
       ▼
┌─────────────┐
│  prepared   │  ← 阶段已批准
└──────┬──────┘
       │ ddd exec
       ▼
┌─────────────┐
│ developing  │  ← 开发中
└──────┬──────┘
       │ 所有阶段完成
       ▼
┌─────────────┐
│  archived   │  ← 已归档
└─────────────┘
```

**阶段状态（per phase）**

每个阶段（phase）独立演进：

```
initialized → developing → verifying → fixing → ready
```

| 状态 | 说明 |
|------|------|
| `initialized` | 阶段初始化 |
| `developing` | AI Agent 开发中 |
| `verifying` | 验证中 |
| `fixing` | 修复中 |
| `ready` | 阶段完成 |

## License

MIT
