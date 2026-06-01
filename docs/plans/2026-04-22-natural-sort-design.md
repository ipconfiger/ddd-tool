# 自然排序修复设计

## 问题

`gen-phase` 命令扫描 `project_docs/phases` 目录时，使用字典序排序文件名：

```rust
phase_files.sort_by_key(|e| e.file_name());
```

当 phase 数量超过 10 时，出现排序错误：`P10-file-upload.md` 排在 `P2-elder.md` 前面。

## 目标

实现自然排序（natural sort），按文件名中的数字数值排序，而非字典序。

## 设计方案

### 排序 key 函数

创建 `extract_sort_key` 函数，从文件名中提取第一个连续数字序列：

```rust
fn extract_sort_key(filename: &OsStr) -> (Option<u32>, &str) {
    let s = filename.to_string_lossy();
    // 提取第一个连续数字
    let num = s.chars()
        .collect::<String>()
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse::<u32>().ok());
    (num, &*s)
}
```

### 排序规则

- 有数字的文件：按数字排序（如 `P10` < `P2`）
- 无数字的文件：`num = None`，按文件名字典序
- 数字相同时：按文件名字典序

### 修改位置

`src/commands/internal.rs` 第 20 行：

```rust
// 之前
phase_files.sort_by_key(|e| e.file_name());

// 之后
phase_files.sort_by_cached_key(|e| extract_sort_key(&e.file_name()));
```

## 验证

| 文件名 | 当前排序 | 期望排序 |
|--------|----------|----------|
| P0-scaffold.md | 1 | 1 |
| P1-auth-user.md | 2 | 2 |
| P2-elder.md | 3 | 5 |
| P10-file-upload.md | 4 | 3 |
| P11-admin.md | 5 | 4 |

## 风险

- 无外部依赖，纯 Rust 实现
- 不修改文件读取逻辑，只改排序
