# State Machine Refactor Design

## Goal

Move all phrase-level state machine logic (get current phase, get next phase, advance to next phase) into `RoadmapState` methods, eliminating scattered logic in commands and fixing the borrow checker error in `confirm_phase.rs`.

## Problem

The current `confirm_phase.rs` has a borrow checker error:

```rust
let current_phase = if let Some(name) = &state.current_phase {
    state.phrases.iter_mut().find(|p| &p.name == name)  // first mutable borrow
} else {
    None
};
let next_phase = if let Some(current) = current_phase {
    if let Some(this_pos) = state.phrases.iter_mut().position(|p| p.name == current.name) {  // second mutable borrow
        state.phrases.get_mut(this_pos + 1usize)  // third mutable borrow
    } else { None }
};
```

Multiple simultaneous mutable borrows of `state.phrases` cause compile errors.

## Design

### New Methods on `RoadmapState`

```rust
impl RoadmapState {
    /// Returns the currently active phase, if one is set.
    pub fn current_phase(&self) -> Option<&Phrase> {
        let name = self.current_phase.as_ref()?;
        self.phrases.iter().find(|p| &p.name == name)
    }

    /// Advances to the next phase in sequence.
    /// Marks the current phase as `STATE_READY`.
    /// Returns the new current phase, or None if already at the end.
    pub fn advance_phase(&mut self) -> Result<Option<&Phrase>> {
        let current_name = match self.current_phase.as_ref() {
            Some(n) => n.clone(),
            None => return Ok(None),
        };

        let current_pos = self.phrases
            .iter()
            .position(|p| p.name == current_name)
            .context("current_phase references missing phrase")?;

        // Mark current as ready
        self.phrases[current_pos].status = STATE_READY.to_string();

        // Advance to next
        if current_pos + 1 < self.phrases.len() {
            let next_name = self.phrases[current_pos + 1].name.clone();
            self.current_phase = Some(next_name);
            return Ok(self.phrases.get(current_pos + 1));
        }

        self.current_phase = None;
        Ok(None)
    }

    pub fn is_all_phases_complete(&self) -> bool {
        self.phrases.iter().all(|p| p.status == STATE_READY)
    }

    /// Initializes the phrases from a list of (name, file) pairs.
    pub fn init_phases_from_files(&mut self, files: Vec<(String, String)>) {
        self.doc_ready = true;
        self.workflow = STATE_PREPARE.to_string();
        self.phrases = files
            .into_iter()
            .enumerate()
            .map(|(idx, (name, file))| Phrase {
                name,
                status: STATE_INIT.to_string(),
                file,
                fixes: vec![],
            })
            .collect();
        self.current_phase = self.phrases.first().map(|p| p.name.clone());
    }
}
```

### Updated `confirm_phase.rs`

```rust
fn do_run() -> Result<()> {
    let mut ctx = DddContext::new()?;

    let mut state = ctx.load_state()?;

    if !state.doc_ready {
        println!("ECHO:请先完成文档准备阶段: 调用 /ddd-accept` 批准开发计划. 停止执行!");
        return Ok(());
    }

    if state.current_phase.is_none() {
        println!("尚未启动开发");
        return Ok(());
    }

    match state.advance_phase()? {
        Some(next) => {
            ctx.save_state(&state)?;
            println!("接下来调用 /ddd-exec 开始实现 {}", next.name);
        }
        None => {
            if state.is_all_phases_complete() {
                println!("全部阶段已经开发完成, 根据 @project_docs/specs/ 目录下的所有的规格文件 和 @project_docs/phases/ 的开发计划作为资料,结合当前实现的代码,进行交叉事实审核,高精度代码评审. 结束后询问是否执行 /ddd-achive 归档此轮开发");
            }
        }
    }
    Ok(())
}
```

### Updated `internal.rs`

Replace direct phrase/field mutation with `state.init_phases_from_files(...)`.

## Files to Change

| File | Change |
|------|--------|
| `src/state/roadmap.rs` | Add 4 methods to `RoadmapState` |
| `src/commands/confirm_phase.rs` | Simplify using new methods, remove borrow errors |
| `src/commands/internal.rs` | Use `init_phases_from_files` |
| `src/commands/prepare.rs` | No change needed |
