# 计划：DocDriven CLI 需求分析与模块拆分

## 需求摘要

根据 `docs/new_spec_v2.md` 文档分析，这是一个 **DocDriven Guide (v2.1 数据驱动版)** 的 Rust CLI 项目规范。核心功能是：

- 基于 `roadmap.json` 声明式状态机的文档驱动开发框架
- 封装核心流命令：`init`, `prepare`, `exec`, `verify`, `fix-plan`, `fix-exec`, `archive`, `report`, `sync`
- 内部状态触发命令：`gen-phrase`, `set-issuse`, `finish-fix`, `finish-phrase`
- 辅助工具命令：`setup`, `help`, `version`, `study`, `resume`

## 接受标准

1. ✅ 读取 `docs/new_spec_v2.md` 完成需求分析
2. ✅ 将需求拆分为独立模块 `.md` 文件，存储到 `@project_docs/specs/`
3. ✅ 按 LLM Wiki 方式连接各模块 spec 文件（索引 + 交叉引用）
4. ✅ 生成 `index.md` 作为入口文档

## 实现步骤

### 步骤 1：需求模块拆分

从 `new_spec_v2.md` 提取以下模块：

| 模块名称 | 文件路径 | 内容描述 |
|---------|---------|---------|
| **spec_core** | `specs/spec_core.md` | 核心机制、状态机定义、workflow/phases/status 语义 |
| **spec_commands** | `specs/spec_commands.md` | 所有命令的详细定义、执行流程、状态校验规则 |
| **spec_data_flow** | `specs/spec_data_flow.md` | 数据定位规范、Prompt 参数映射、JSON Schema |
| **spec_state_machine** | `specs/spec_state_machine.md` | 状态流转图、workflow/phases/fixes 状态机定义 |
| **spec_engineering** | `specs/spec_engineering.md` | 工程规范、最佳实践、安全约束 |

### 步骤 2：生成 LLM Wiki 索引

创建 `specs/index.md` 作为入口文件，包含：
- 项目概述
- 各模块文档的链接关系
- 命令索引表
- 状态机快速参考

### 步骤 3：验证文档完整性

- 检查所有 spec 文件存在
- 验证 Wiki 链接有效性
- 确认 roadmap.json 的字段与 spec 描述一致

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|-----|------|---------|
| spec 拆分粒度不当 | 后续开发耦合 | 按"命令族"和"数据模型"两大维度拆分 |
| Wiki 链接断裂 | 文档不可读 | 使用 `@project_docs/` 别名统一路径 |

## 验证步骤

1. `ls project_docs/specs/` → 确认 5 个 spec 文件 + index.md 存在
2. 检查 index.md 包含对所有 spec 文件的引用
3. 确认 roadmap.json Schema 与 spec_core.md 描述一致

---

*计划创建时间：2026-04-19*
