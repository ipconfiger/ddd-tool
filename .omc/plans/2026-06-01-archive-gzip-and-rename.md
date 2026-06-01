# Plan: Archive 命令改为 gzip 归档 + 全代码 `phrases` → `phases` 重命名

**Mode**: Direct（用户已给出详细需求，跳过 Interview）
**Date**: 2026-06-01
**Branch**: main

---

## 1. Requirements Summary

### 1.1 功能变更
将 `ddd-tool archive` 命令中"移动 specs/phrases 目录"的两步操作，改为：
1. 将 `project_docs/specs/` 目录打包为 `specs.tar.gz`
2. 将 `project_docs/phases/` 目录打包为 `phases.tar.gz`
3. 两个 `.tar.gz` 文件均输出到当次归档目录 `<archives>/<YYYYMMDD-N>/` 下
4. 归档完成后 **清空**（移除所有内容、保留空目录）`project_docs/specs/` 与 `project_docs/phases/`

### 1.2 命名规范统一
将代码中所有标识符、字符串、注释、测试中出现的 `phrases`/`Phrase`/`phrase`（与"开发阶段"语义相关者）统一重命名为 `phases`/`Phase`/`phase`。
- 包含 JSON 字段名 `phrases`（影响序列化兼容性 → 见风险 5.1）
- 包含目录名 `phrases` → `phases`（磁盘上现在已经是 `phases/`，`prepare.rs` 写错为 `phrases/`）

### 1.3 现有不一致基线
- `src/commands/internal.rs:21` 读 `project_docs/phases/`（磁盘实际目录）
- `src/commands/prepare.rs:47` 写 `project_docs/phrases/`（错！）
- `src/commands/archive.rs:51` 重命名 `phrases`（错！）

---

## 2. 关键设计决策（用户已确认）

| # | 决策 | 选择 | 理由 |
|---|------|------|------|
| D1 | 归档格式 | `.tar.gz`（tar+gz 流式） | 目录必须先 tar 再 gzip；保留目录结构与文件名；增量恢复可行 |
| D2 | 输出文件数 | **1 个**（`archive-<YYYYMMDD-N>.tar.gz`，内含 `specs/` + `phases/`） | **用户答复 Q1** |
| D3 | Crate 依赖 | `tar = "0.4"` + `flate2 = "1.0"` | Rust 生态最稳定组合；flate2 默认 zlib 后端 |
| D4 | "清空" 语义 | `remove_dir_all` 后 `create_dir_all` | 与 `prepare.rs:46-51` 行为一致；空目录作为下一轮 prepare 锚点 |
| D5 | `Phrase` 结构体重命名 | `Phrase` → `Phase` | **用户答复 Q2**；与字段 `phases` 命名一致 |
| D6 | Prompt 模板 `{Phrase Name}` | 改为 `{Phase Name}` | **用户答复 Q3**；同步改 `PromptParams::with_phrase_name` → `with_phase_name`，字段 `phrase_name` → `phase_name` |
| D7 | 归档子目录命名 | `<YYYYMMDD>-<N>` 沿用 | 不破坏现有目录计数逻辑；新归档文件名 `archive-<YYYYMMDD-N>.tar.gz` |
| D8 | 目录名 `phrases` 磁盘错误 | `prepare.rs:47` `phrases` → `phases`（同步 disk 真相） | **用户答复 Q3** "同步修改设计其他部分"，统一目录命名 |

---

## 3. Acceptance Criteria

### 3.1 archive 命令功能
- [ ] **AC-1** 当 `state.phases` 全部为 `finished` 时，`ddd archive` 在 `project_docs/archives/<YYYYMMDD-N>/` 下生成单个 `archive-<YYYYMMDD-N>.tar.gz`，内部包含 `specs/` 与 `phases/` 两个目录
- [ ] **AC-2** 解压 `archive-<YYYYMMDD-N>.tar.gz` 后可还原出原 `project_docs/specs/` 完整目录树
- [ ] **AC-3** 解压 `archive-<YYYYMMDD-N>.tar.gz` 后可还原出原 `project_docs/phases/` 完整目录树
- [ ] **AC-4** 归档完成后 `project_docs/specs/` 与 `project_docs/phases/` 存在但为空目录
- [ ] **AC-5** 当 specs/ 或 phases/ 不存在时，跳过对应归档但不报错（沿用当前 `if src.exists()` 行为）
- [ ] **AC-6** `roadmap.json` 重置为初始状态（`workflow: "init"`, `current_phase: None`, `phases: []`）
- [ ] **AC-7** 终端输出 `✅ 项目已归档到: @project_docs/archives/<date>/`，且提示中包含 `archive-<YYYYMMDD-N>.tar.gz`

### 3.2 命名重命名
- [ ] **AC-8** 仓库内 `src/` 目录下，标识符、字符串字面量、注释中均不再出现 `phrases` 或 `Phrase`（搜索：`rg -n "phras" src/` 应无结果，**但** `anem` 等无关拼写错误不在范围）
- [ ] **AC-9** `RoadmapState` 序列化 JSON 字段由 `phrases` 改为 `phases`（破坏性变更 → 风险 5.1）
- [ ] **AC-10** 所有 prompt 模板占位符从 `{Phrase Name}` 改为 `{Phase Name}`，配套 `PromptParams::with_phrase_name` → `with_phase_name`，`phrase_name` 字段 → `phase_name`
- [ ] **AC-11** `prepare.rs` 写目录路径从 `phrases` 修正为 `phases`（与磁盘一致）
- [ ] **AC-12** 测试用例同步更新（`src/state/roadmap.rs` 内 30+ 处 `state.phrases`、`Phrase {...}`、`@project_docs/phrases/phrase0.md` 全部替换）

### 3.3 验证闭环
- [ ] **AC-13** `cargo build` 无 warning（同时清掉既有 dead_code 警告：`Phrase` 字段移除后 `WORKFLOW_INIT/WORKFLOW_READY/WORKFLOW_ARCHIVED/PHASE_VERIFYING` 等未使用常量、`setup.rs::backup_dir/backup_file`、`context::with_root`、`exec.rs` 未使用 import、`internal.rs::state` 模块等）
- [ ] **AC-14** `cargo test` 全部通过
- [ ] **AC-15** 手工 e2e：在一个项目目录中 `prepare → accept → exec → verify → confirm ×N → archive`，验证：
  - 归档目录存在且含两个 `.tar.gz`
  - 原 specs/、phases/ 为空
  - `roadmap.json` 已重置
  - 解压 `phases.tar.gz` 可恢复所有 phase 文件

---

## 4. Implementation Steps

### Step 1：依赖与目录准备
**文件**：`Cargo.toml`
```toml
[dependencies]
flate2 = "1.0"
tar = "0.4"
```
执行 `cargo check` 确认依赖可解析。

---

### Step 2：实现 `archive_gzip` 工具函数
**文件**：`src/commands/archive.rs`（重写）

```rust
use crate::commands::{DddContext, ArchiveCmd};
use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::Path;
use tar::Builder;

pub fn run(_cmd: ArchiveCmd) {
    if let Err(e) = do_run() {
        eprintln!("错误: {}", e);
    }
}

fn do_run() -> Result<()> {
    let ctx = DddContext::new()?;
    let state = ctx.load_state()?;

    // 1. 校验所有 phases 已完成
    let unfinished: Vec<_> = state.phases.iter()
        .filter(|p| p.status != "finished")
        .collect();
    if !unfinished.is_empty() {
        println!("请先完成所有开发阶段:");
        for p in &unfinished {
            println!("  - {} (状态: {})", p.name, p.status);
        }
        return Ok(());
    }

    // 2. 创建归档目录
    let archives_dir = ctx.project_root.join("project_docs").join("archives");
    fs::create_dir_all(&archives_dir)?;
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let idx = fs::read_dir(&archives_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            e.path().file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&today))
                .unwrap_or(false)
        })
        .count();
    let archive_name = format!("{}-{}", today, idx);
    let archive_path = archives_dir.join(&archive_name);
    fs::create_dir_all(&archive_path)?;

    // 3. gzip 归档 specs 与 phases
    let project_docs = ctx.project_root.join("project_docs");
    let dirs_to_archive = [("specs", "specs.tar.gz"), ("phases", "phases.tar.gz")];
    for (dir_name, tar_gz_name) in dirs_to_archive {
        let src = project_docs.join(dir_name);
        if src.exists() {
            let dst = archive_path.join(tar_gz_name);
            gzip_dir(&src, &dst)
                .with_context(|| format!("归档 {} 失败", dir_name))?;
        }
    }

    // 4. 清空 specs/ 与 phases/
    for (dir_name, _) in dirs_to_archive {
        let dir = project_docs.join(dir_name);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
    }

    // 5. 重置 roadmap.json
    let initial_state = crate::state::RoadmapState::new();
    ctx.save_state(&initial_state)?;

    println!("✅ 项目已归档到: @project_docs/archives/{}/", archive_name);
    println!("  - specs.tar.gz");
    println!("  - phases.tar.gz");
    println!("roadmap.json 已重置为初始状态。");
    Ok(())
}

/// 将 src 目录递归打包为 .tar.gz 写入 dst
fn gzip_dir(src: &Path, dst: &Path) -> Result<()> {
    let file = File::create(dst)
        .with_context(|| format!("创建文件失败: {}", dst.display()))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);
    builder.append_dir_all(".", src)
        .with_context(|| format!("打包目录失败: {}", src.display()))?;
    builder.into_inner()
        .context("gzip 编码失败")?
        .finish()
        .context("关闭文件失败")?;
    Ok(())
}
```

---

### Step 3：状态层 `phrases` → `phases` 重命名
**文件**：`src/state/roadmap.rs`、`src/state/mod.rs`

`roadmap.rs` 内必改（行号已确认）：
- `:32` `pub struct Phrase` → `pub struct Phase`
- `:46` `pub phrases: Vec<Phrase>` → `pub phases: Vec<Phase>`
- `:57` `phrases: vec![]` → `phases: vec![]`
- `:62` 返回类型 `Option<&Phrase>` → `Option<&Phase>`
- `:64, 66, 78, 84, 95, 99, 114-126, 135, 142, 145, 152, 164, 346, 353, 356, 360, 368, 380, 382, 392, 420, 442, 445, 454, 457, 477, 480, 490-492, 504, 507, 520, 537-541, 547-549, 554, 561, 569, 575, 580, 583-584, 604, 609, 616, 623, 635, 641, 646, 652, 659, 665, 670, 676, 688-689, 706` 等所有 `state.phrases` / `Phrase {...}` / 字符串 `"Phrase0"` / `"Phrase1"` 改为对应 `phases` / `Phase {...}` / `"Phase0"` / `"Phase1"`
- `:349, 371, 393, 421, 445, 457, 480, 507` 中字符串 `"@project_docs/phrases/phrase0.md"` → `"@project_docs/phases/0.md"`（与 `internal.rs:37,39` 中已用的命名一致：`Phase{n}` + `@project_docs/phases/{filename}`）
- 函数名 `init_phrases_from_files` → `init_phases_from_files`（已是 phases 形式，但函数体内字段访问要改）
- `is_all_phases_complete()` 函数体访问 `self.phrases` → `self.phases`
- `fetch_current_phase()`、`advance_phase()`、`set_phase_dev()`、`set_phase_finished()`、`is_current_init()` 函数体全部访问 `self.phrases` 改 `self.phases`

`state/mod.rs:4`：
- `pub use roadmap::{RoadmapState, RoadmapStore, Phrase};` → `pub use roadmap::{RoadmapState, RoadmapStore, Phase};`

⚠️ **JSON 兼容**：`#[derive(Serialize, Deserialize)]` 的 `phrases` 字段重命名后，旧版 `roadmap.json` 会被反序列化为空数组（serde 默认行为）。需决定：
- 方案 A：接受破坏（用户要求"严格重命名"，视为可接受破坏性变更）
- 方案 B：用 `#[serde(alias = "phrases")]` 兼容（**不推荐**，违反用户意图）

采用 **方案 A**，在 plan 中明确告知。

---

### Step 4：命令层重命名
**文件清单与改动**：

| 文件 | 改动 |
|------|------|
| `src/commands/mod.rs:29` | 注释 `/// 扫描 phrases 目录，生成 phrases 数组` → `/// 扫描 phases 目录，生成 phases 数组` |
| `src/commands/prepare.rs:46-51` | 变量名 `phrases_dir` → `phases_dir`；路径 `phrases` → `phases`（修历史错）；注释 `// 清空 phrases 目录` → `// 清空 phases 目录` |
| `src/commands/prepare.rs:26` | 字符串中的 `@project_docs/phases/` 已是正确拼写（无需改），但**PromptPrompt** 不在范围 |
| `src/commands/internal.rs:16,20,32,57` | 注释 `phrases` → `phases`；变量名 `phrases_dir` → `phases_dir`；`state.phrases.len()` → `state.phases.len()` |
| `src/commands/internal.rs:53` | `state.init_phrases_from_files(files)` → `state.init_phases_from_files(files)`（仅当函数名被重命名） |
| `src/commands/verify.rs:39` | `state.phrases.iter_mut()` → `state.phases.iter_mut()` |
| `src/commands/report.rs:41,56,57` | `state.phrases` → `state.phases` |
| `src/commands/final_verify.rs:37` | `state.is_all_phases_complete()` 已是 phases 拼写（无需改） |
| `src/commands/setup.rs:8,9` | COMMAND_DESC 字符串 `"Prepare phrases from specs"` → `"Prepare phases from specs"`；`"Accept development plan, init phrases"` → `"Accept development plan, init phases"` |
| `src/commands/exec.rs:32` | 注释 `phases[0]` 已是正确拼写（无需改）；`:46` `is_all_phases_complete()` 调用不变 |
| `src/commands/exec.rs:4` | 清理诊断警告：移除未使用的 `PHASE_DEV, PHASE_INIT, WORKFLOW_DEV` import |
| `src/commands/setup.rs:91` | 修复诊断警告：未使用变量 `desc` → `_desc` |
| `src/commands/archive.rs` | 同步重命名（已在 Step 2 完成） |

---

### Step 5：Prompt 层重命名
**文件**：`src/prompts/mod.rs`

- 字段 `phrase_name: Option<String>` → `phase_name: Option<String>`（line 7）
- 方法 `with_phrase_name` → `with_phase_name`（line 32）
- 替换占位符 `"{Phrase Name}"` → `"{Phase Name}"`（line 64, 90）
- 测试调用 `with_phrase_name(...)` → `with_phase_name(...)`（line 95）
- 测试模板 `{Phrase Name}` → `{Phase Name}`（line 90）

---

### Step 6：清理诊断警告（顺手）
- `src/state/roadmap.rs:10-13,17` 删除未使用常量（`WORKFLOW_INIT`, `WORKFLOW_READY`, `WORKFLOW_ARCHIVED`, `PHASE_VERIFYING`）—— 这些不在状态机使用路径中
- `src/state/roadmap.rs:21-22` `WORKFLOW_STATES` 与 `PHASE_STATES` 看是否被使用（`validate()` 用到了 `WORKFLOW_STATES` 和 `PHASE_STATES`，需保留；但需确认 `PHASE_VERIFYING` 没在数组里）
- `src/commands/mod.rs:6` `DOCUMENT_STAGE` 常量未使用，删除
- `src/commands/mod.rs:4` 检查 `Phrase` 重新导出是否被外部使用（`exec.rs` 等用 `state::roadmap::PHASE_*` 路径直接访问，未用 `crate::state::Phrase`，可保留 `Phase` 重导出）
- `src/commands/setup.rs:228,245` 删除 `backup_dir` / `backup_file` 死函数
- `src/commands/audit.rs:40` 删除 `render` 死函数
- `src/commands/context.rs:24` 删除 `with_root` 死方法

---

### Step 7：编译与测试
1. `cargo build` —— 0 warning
2. `cargo test` —— 全部通过
3. `cargo clippy --all-targets` —— 0 warning

---

## 5. Risks and Mitigations

### 5.1 破坏性 JSON 兼容（高）
**风险**：`RoadmapState.phrases` 字段重命名后，旧版本 `roadmap.json` 中 `phrases` 数组将被反序列化为 `phases: Vec::default()`（空），导致当前进度丢失。
**缓解**：
- 计划阶段已确认采用方案 A（破坏性）
- CHANGELOG / README 须标注 **"v0.1.x → v0.2.0: 破坏性升级，需先 `ddd archive` 旧数据再升级"**（本任务范围内不写 CHANGELOG，留待后续任务）
- 执行顺序：先完成 archive 命令的 gzip 改造，再用新 archive 清理旧状态，再升级字段

### 5.2 跨卷 `rename` / 归档失败回滚（中）
**风险**：当前 `fs::rename` 是原子的；新流程是"先 gzip 写新文件，再 remove_dir_all 源目录"。中间失败时可能留下半成品归档。
**缓解**：
- gzip 写入使用临时文件 `.tmp` → 成功后 `rename` 为最终名
- 源目录清理放在所有 gzip 都成功后执行
- 任一步骤失败时，错误向上传播但不删除已写入的 `.tar.gz`（避免误删可恢复数据）

### 5.3 目录名 `phases` 误改（低）
**风险**：`prepare.rs` 写 `phrases/` 是历史 bug（与磁盘不一致），重命名过程中若将本任务误判为 "改字段名" 而不改目录路径，会引入新问题。
**缓解**：
- Step 4 显式列出 `prepare.rs:47` 路径从 `phrases` 改为 `phases`（与 `internal.rs:21` 对齐）
- AC-11 单独标注此验收点

### 5.4 `Phrase` 类型重命名导致 `pub use` 失效（中）
**风险**：`src/state/mod.rs:4` 重导出 `Phrase` 被外部模块 `use crate::state::Phrase` 使用。改名后须同步更新。
**缓解**：
- `rg "use crate::state::Phrase"` 已确认仅在测试与 `roadmap.rs` 内部使用（见 grep 结果）
- `mod.rs:4` 改为 `pub use roadmap::{RoadmapState, RoadmapStore, Phase};`
- `src/prompts/mod.rs` 提示未直接引用 `Phrase` 类型（通过 `with_phrase_name` 字符串方法）

### 5.5 测试用例 `Phrase0` / `Phrase1` 字面量（低）
**风险**：测试中 `"Phrase0"` / `"Phrase1"` 是字面量，与类型 `Phrase` 重命名独立但语义相关。
**缓解**：与结构体重命名一起改为 `"Phase0"` / `"Phase1"`（D5 决策）

### 5.6 大目录内存峰值（低）
**风险**：`tar::Builder::append_dir_all` 会先 walk 整个目录树，然后写入。
**缓解**：
- 对一般项目（specs/、phases/ 几个 .md 文件）无压力
- 若未来 phases/ 巨大，可改用 `append_file` 流式；本任务范围不预先优化（YAGNI）

### 5.7 双归档产物 vs 单归档（设计）
**风险**：D2 决策"两个独立 tar.gz"若用户实际期望"一个合并 tar.gz"会与意图冲突。
**缓解**：D2 在 §2 中已标注为"可调整"，执行前用户可指明偏好（默认按 D2 走）。

---

## 6. Verification Steps

### 6.1 静态验证
```bash
# 命名重命名完整性
rg -n "phras" src/                        # 期望：除 "phrase" 作为英文单词的注释外无残留
rg -n "Phrase" src/                        # 期望：无残留（结构体已改名）
rg -n "phrase_name" src/                   # 期望：无残留

# 路径一致性
rg -n "project_docs/phrases" src/          # 期望：无结果（应统一为 phases）
rg -n "project_docs/phases" src/           # 期望：命中处均为 phases
```

### 6.2 编译验证
```bash
cargo clean
cargo build                                # 期望：0 error, 0 warning
cargo build --tests                        # 期望：0 error, 0 warning
cargo clippy --all-targets -- -D warnings  # 期望：通过
```

### 6.3 单元 / 集成测试
```bash
cargo test                                 # 期望：全部通过
cargo test state::roadmap                  # 期望：roadmap 测试通过（验证 Phase 字段生效）
cargo test prompts                         # 期望：模板渲染测试通过（验证 {Phase Name} 替换生效）
cargo test commands::archive               # 期望：新 archive 测试通过（如新增）
```

### 6.4 e2e 验证
```bash
# 准备：构造一个含 specs/、phases/ 的项目目录
mkdir -p /tmp/ddd-e2e/project_docs/{specs,phases}
echo "# spec" > /tmp/ddd-e2e/project_docs/specs/spec1.md
echo "# phase 1" > /tmp/ddd-e2e/project_docs/phases/1_foo.md
echo "# phase 2" > /tmp/ddd-e2e/project_docs/phases/2_bar.md
# 写一个最小 roadmap.json
cat > /tmp/ddd-e2e/project_docs/roadmap.json <<'JSON'
{
  "version": "1.0.0",
  "updated_at": "2026-06-01T00:00:00Z",
  "workflow": "dev",
  "current_phase": null,
  "doc_ready": true,
  "phases": [
    {"name":"Phase1","status":"finished","file":"@project_docs/phases/1_foo.md","fixes":[]},
    {"name":"Phase2","status":"finished","file":"@project_docs/phases/2_bar.md","fixes":[]}
  ]
}
JSON
cd /tmp/ddd-e2e && ddd-tool archive
# 验证：
ls project_docs/archives/*/                # 期望：specs.tar.gz, phases.tar.gz
file project_docs/archives/*/specs.tar.gz  # 期望：gzip compressed data
tar -tzf project_docs/archives/*/specs.tar.gz  # 期望：含 ./spec1.md
tar -tzf project_docs/archives/*/phases.tar.gz # 期望：含 ./1_foo.md, ./2_bar.md
ls project_docs/specs/                     # 期望：空目录
ls project_docs/phases/                    # 期望：空目录
cat project_docs/roadmap.json              # 期望：phases: []
```

### 6.5 回归验证
```bash
# 验证 prep 阶段仍能正常生成 phases/（含 index.md）
# 验证 accept 仍能扫描 phases/ 生成 roadmap
# 验证 exec/verify/confirm 链路未受影响
```

---

## 7. 实施顺序与并行建议

| 顺序 | 步骤 | 依赖 | 可并行 |
|------|------|------|--------|
| 1 | Step 1：加依赖 | — | — |
| 2 | Step 2：archive.rs 重写 + gzip 工具 | Step 1 | — |
| 3 | Step 3：state 重命名 | — | 与 Step 4-5 并行 |
| 4 | Step 4：commands 重命名 | Step 3（state 类型先就绪） | — |
| 5 | Step 5：prompts 重命名 | — | 与 Step 4 并行 |
| 6 | Step 6：清理 dead code 警告 | Step 3-5 完成后 | — |
| 7 | Step 7：build + test | Step 2-6 | — |

---

## 8. Out of Scope（明确不做）

- 不写 CHANGELOG（用户未要求）
- 不更新 `docs/new_spec_v2.md` 与 `docs/plans/*.md`（历史文档留作回溯证据）
- 不更新 `README.md`（除非用户后续要求；本任务只关注代码）
- 不重写 `audit.rs` `render` 函数、`setup.rs` 备份函数、`context.rs::with_root`（诊断要求是删，不用替换实现）
- 不引入增量归档 / 压缩级别 / 加密等高级特性（YAGNI）
- 不修改 `Cargo.toml` 的 version 字段（破坏性变更需要 bump 由用户/后续任务决策）

---

## 9. Open Questions（已确认）

1. **D2 归档文件数**：✅ **一个** `archive-<YYYYMMDD-N>.tar.gz`（内含 specs/、phases/）
2. **D5 `Phrase` → `Phase`**：✅ 结构体、字段、字面量 `Phrase0/1` 全部改 `Phase`
3. **D6 Prompt 占位符**：✅ 同步改 `{Phrase Name}` → `{Phase Name}`、`phrase_name` → `phase_name`；同时**统一磁盘目录名**（`prepare.rs:47` 错的 `phrases` → `phases`）
