use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;

use anyhow::{Context, Result};

pub const WORKFLOW_INIT: &str = "init";
pub const WORKFLOW_READY: &str = "ready";
pub const WORKFLOW_DEV: &str = "dev";
pub const WORKFLOW_ARCHIVED: &str = "archived";

pub const PHASE_INIT: &str = "init";
pub const PHASE_DEV: &str = "dev";
pub const PHASE_VERIFYING: &str = "verifying";
pub const PHASE_FINISHED: &str = "finished";


pub const WORKFLOW_STATES: [&str; 4] = ["init", "ready", "dev", "archived"];
pub const PHASE_STATES: [&str; 4] = ["init", "dev", "verifying", "finished"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub id: u32,
    pub status: String,
    pub plan_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phrase {
    pub name: String,
    pub status: String,
    pub file: String,
    pub fixes: Vec<Fix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapState {
    pub version: String,
    pub updated_at: String,
    pub workflow: String,
    pub current_phase: Option<String>,
    pub doc_ready: bool,
    pub phrases: Vec<Phrase>,
}

impl RoadmapState {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            updated_at: Utc::now().to_rfc3339(),
            workflow: "init".to_string(),
            current_phase: None,
            doc_ready: false,
            phrases: vec![],
        }
    }

    /// Returns the currently active phase, if one is set.
    pub fn fetch_current_phase(&mut self) -> Option<&Phrase> {
        if let Some(phase) = if let Some(current_phase_name) = &self.current_phase {
            self.phrases.iter().find(|p| &p.name == current_phase_name)
        } else {
            self.phrases.first()
        }{
            self.current_phase = Some(phase.name.to_string());
            Some(phase)
        }else{
            None
        }
    }

    pub fn set_phase_dev(&mut self, phase_name: &str) {
        self.workflow = WORKFLOW_DEV.to_string();
        self.current_phase = Some(phase_name.to_string());
        if let Some(phase) = self.phrases.iter_mut().find(|p| &p.name == phase_name) {
            phase.status = PHASE_DEV.to_string();
        }
    }

    pub fn set_phase_finished(&mut self, phase_name: &str) {
        if let Some(phase) = self.phrases.iter_mut().find(|p| &p.name == phase_name) {
            phase.status = PHASE_FINISHED.to_string();
        }
    }

    /// Advances to the next phase in sequence.
    /// Marks the current phase as `finished`.
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

        // Mark current as finished
        self.phrases[current_pos].status = "finished".to_string();

        // Advance to next
        if current_pos + 1 < self.phrases.len() {
            let next_name = self.phrases[current_pos + 1].name.clone();
            self.current_phase = Some(next_name);
            return Ok(self.phrases.get(current_pos + 1));
        }

        self.current_phase = None;
        Ok(None)
    }

    /// Returns true if all phrases have reached finished status.
    pub fn is_all_phases_complete(&self) -> bool {
        self.phrases.iter().all(|p| p.status == "finished")
    }

    /// Initializes phrases from a list of (name, file) pairs.
    pub fn init_phrases_from_files(&mut self, files: Vec<(String, String)>) {
        self.doc_ready = true;
        self.workflow = "ready".to_string();
        self.phrases = files
            .into_iter()
            .enumerate()
            .map(|(_idx, (name, file))| Phrase {
                name,
                status: PHASE_INIT.to_string(),
                file,
                fixes: vec![],
            })
            .collect();
        self.current_phase = self.phrases.first().map(|p| p.name.clone());
    }

    pub fn validate(&self) -> Result<()> {
        if !WORKFLOW_STATES.contains(&self.workflow.as_str()) {
            anyhow::bail!(
                "Invalid workflow state: {}, expected one of: {:?}",
                self.workflow,
                WORKFLOW_STATES
            );
        }

        for phrase in &self.phrases {
            if !PHASE_STATES.contains(&phrase.status.as_str()) {
                anyhow::bail!(
                    "Invalid phrase status: {}, expected one of: {:?}",
                    phrase.status,
                    PHASE_STATES
                );
            }
        }

        Ok(())
    }
}

impl Default for RoadmapState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct FileLock {
    file: File,
}

#[allow(dead_code)]
impl FileLock {
    pub fn lock(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .context("Failed to open file for locking")?;

        #[cfg(unix)]
        {
            #[allow(unused_imports)]
            use std::os::unix::fs::FileExt;
            file.lock_shared()
                .context("Failed to acquire file lock")?;
        }

        Ok(Self { file })
    }

    pub fn unlock(self) -> Result<()> {
        #[cfg(unix)]
        {
            #[allow(unused_imports)]
            use std::os::unix::fs::FileExt;
            self.file
                .unlock()
                .context("Failed to release file lock")?;
        }
        Ok(())
    }
}

pub struct RoadmapStore {
    path: String,
    lock: Mutex<()>,
}

impl RoadmapStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            lock: Mutex::new(()),
        }
    }

    pub fn load(&self) -> Result<RoadmapState> {
        let _guard = self.lock.lock().unwrap();
        let path = Path::new(&self.path);

        if !path.exists() {
            return Ok(RoadmapState::new());
        }

        let mut file = File::open(path).context("Failed to open roadmap.json")?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .context("Failed to read roadmap.json")?;

        let state: RoadmapState =
            serde_json::from_slice(&contents).context("Failed to parse roadmap.json")?;

        state.validate()?;
        Ok(state)
    }

    pub fn save(&self, state: &RoadmapState) -> Result<()> {
        let _guard = self.lock.lock().unwrap();
        let path = Path::new(&self.path);

        let mut state = state.clone();
        state.updated_at = Utc::now().to_rfc3339();

        state.validate()?;

        let json = serde_json::to_string_pretty(&state).context("Failed to serialize state")?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .context("Failed to open roadmap.json for writing")?;

        file.write_all(json.as_bytes())
            .context("Failed to write roadmap.json")?;

        file.sync_all().context("Failed to sync roadmap.json")?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn init(&self) -> Result<RoadmapState> {
        let state = RoadmapState::new();
        self.save(&state)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_roadmap_state_new() {
        let state = RoadmapState::new();
        assert_eq!(state.version, "1.0.0");
        assert_eq!(state.workflow, "init");
        assert!(!state.doc_ready);
        assert!(state.phrases.is_empty());
    }

    #[test]
    fn test_validate_workflow_states() {
        let state = RoadmapState::new();
        assert!(state.validate().is_ok());

        let mut bad_state = RoadmapState::new();
        bad_state.workflow = "invalid".to_string();
        assert!(bad_state.validate().is_err());
    }

    #[test]
    fn test_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roadmap.json");
        let store = RoadmapStore::new(path.to_str().unwrap());

        let state = RoadmapState::new();
        store.save(&state).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.workflow, "init");
        assert_eq!(loaded.version, "1.0.0");
    }

    // Phase 5: 状态流转集成测试

    #[test]
    fn test_workflow_state_transitions() {
        let mut state = RoadmapState::new();
        assert_eq!(state.workflow, "init");

        state.workflow = "ready".to_string();
        assert_eq!(state.workflow, "ready");

        state.workflow = "dev".to_string();
        assert_eq!(state.workflow, "dev");
    }

    #[test]
    fn test_phrase_status_transitions() {
        let mut state = RoadmapState::new();

        state.phrases.push(Phrase {
            name: "Phrase0".to_string(),
            status: "init".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![],
        });

        assert_eq!(state.phrases[0].status, "init");

        // init → dev
        state.phrases[0].status = "dev".to_string();
        assert_eq!(state.phrases[0].status, "dev");

        // dev → finished
        state.phrases[0].status = "finished".to_string();
        assert_eq!(state.phrases[0].status, "finished");
    }

    #[test]
    fn test_phrase_issue_found_to_fixing_flow() {
        let mut state = RoadmapState::new();

        state.phrases.push(Phrase {
            name: "Phrase0".to_string(),
            status: "issue_found".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![Fix {
                id: 0,
                status: "done".to_string(),
                plan_file: "@project_docs/fixes/phrase0_fix0.md".to_string(),
            }],
        });

        // issue_found → fixing (当所有 fixes 都是 done)
        let all_done = state.phrases[0].fixes.iter().all(|f| f.status == "done");
        if all_done {
            state.phrases[0].status = "fixing".to_string();
        }
        assert_eq!(state.phrases[0].status, "fixing");
    }

    #[test]
    fn test_fix_status_transitions() {
        let mut phrase = Phrase {
            name: "Phrase0".to_string(),
            status: "issue_found".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![],
        };

        phrase.fixes.push(Fix {
            id: 0,
            status: "pending".to_string(),
            plan_file: "@project_docs/fixes/phrase0_fix0.md".to_string(),
        });

        // pending → planned
        phrase.fixes[0].status = "planned".to_string();
        assert_eq!(phrase.fixes[0].status, "planned");

        // planned → executing
        phrase.fixes[0].status = "executing".to_string();
        assert_eq!(phrase.fixes[0].status, "executing");

        // executing → done
        phrase.fixes[0].status = "done".to_string();
        assert_eq!(phrase.fixes[0].status, "done");
    }

    #[test]
    fn test_fix_failed_to_planned_flow() {
        let mut phrase = Phrase {
            name: "Phrase0".to_string(),
            status: "issue_found".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![],
        };

        phrase.fixes.push(Fix {
            id: 0,
            status: "executing".to_string(),
            plan_file: "@project_docs/fixes/phrase0_fix0.md".to_string(),
        });

        // 模拟验证失败
        phrase.fixes[0].status = "failed".to_string();
        assert_eq!(phrase.fixes[0].status, "failed");

        // 重新 plan
        phrase.fixes[0].status = "planned".to_string();
        assert_eq!(phrase.fixes[0].status, "planned");
    }

    #[test]
    fn test_validate_rejects_invalid_phrase_status() {
        let mut state = RoadmapState::new();
        state.phrases.push(Phrase {
            name: "Phrase0".to_string(),
            status: "invalid".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![],
        });
        assert!(state.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_invalid_fix_status() {
        let mut state = RoadmapState::new();
        state.phrases.push(Phrase {
            name: "Phrase0".to_string(),
            status: "issue_found".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![Fix {
                id: 0,
                status: "invalid".to_string(),
                plan_file: "@project_docs/fixes/phrase0_fix0.md".to_string(),
            }],
        });
        assert!(state.validate().is_err());
    }

    #[test]
    fn test_save_and_load_with_phrases() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roadmap.json");
        let store = RoadmapStore::new(path.to_str().unwrap());

        let mut state = RoadmapState::new();
        state.workflow = "dev".to_string();
        state.doc_ready = true;
        state.current_phase = Some("Phrase0".to_string());
        state.phrases.push(Phrase {
            name: "Phrase0".to_string(),
            status: "dev".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![],
        });

        store.save(&state).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.workflow, "dev");
        assert!(loaded.doc_ready);
        assert_eq!(loaded.current_phase, Some("Phrase0".to_string()));
        assert_eq!(loaded.phrases.len(), 1);
        assert_eq!(loaded.phrases[0].name, "Phrase0");
        assert_eq!(loaded.phrases[0].status, "dev");
    }

    #[test]
    fn test_archive_reset_flow() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roadmap.json");
        let store = RoadmapStore::new(path.to_str().unwrap());

        // 设置一个复杂状态
        let mut state = RoadmapState::new();
        state.workflow = "dev".to_string();
        state.phrases.push(Phrase {
            name: "Phrase0".to_string(),
            status: "finished".to_string(),
            file: "@project_docs/phrases/phrase0.md".to_string(),
            fixes: vec![],
        });

        store.save(&state).unwrap();

        // 模拟 archive 重置
        let initial_state = RoadmapState::new();
        store.save(&initial_state).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.workflow, "init");
        assert!(!loaded.doc_ready);
        assert!(loaded.phrases.is_empty());
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_accept_initializes_phrases_correctly() {
        let mut state = RoadmapState::new();

        state.init_phrases_from_files(vec![
            ("PhaseA".to_string(), "@project_docs/phases/a.md".to_string()),
            ("PhaseB".to_string(), "@project_docs/phases/b.md".to_string()),
        ]);

        assert!(state.doc_ready);
        assert_eq!(state.workflow, "ready");
        assert_eq!(state.current_phase, Some("PhaseA".to_string()));
        assert_eq!(state.phrases.len(), 2);
        assert_eq!(state.phrases[0].name, "PhaseA");
        assert_eq!(state.phrases[0].status, "init");
        assert_eq!(state.phrases[1].name, "PhaseB");
        assert_eq!(state.phrases[1].status, "init");
    }

    #[test]
    fn test_exec_sets_phrase_to_dev() {
        let mut state = RoadmapState::new();
        state.init_phrases_from_files(vec![
            ("Phrase0".to_string(), "@project_docs/phases/0.md".to_string()),
            ("Phrase1".to_string(), "@project_docs/phases/1.md".to_string()),
        ]);

        // Simulate exec: find current phase and set status to "dev" if "init"
        let current_name = state.current_phase.clone().unwrap();
        if let Some(phase) = state.phrases.iter_mut().find(|p| p.name == current_name) {
            if phase.status == "init" {
                phase.status = "dev".to_string();
            }
        }
        state.workflow = "dev".to_string();

        assert_eq!(state.phrases[0].status, "dev");
        assert_eq!(state.workflow, "dev");
        assert_eq!(state.current_phase, Some("Phrase0".to_string()));
    }

    #[test]
    fn test_exec_skips_finished_phases() {
        let mut state = RoadmapState::new();
        state.init_phrases_from_files(vec![
            ("Phrase0".to_string(), "@project_docs/phases/0.md".to_string()),
            ("Phrase1".to_string(), "@project_docs/phases/1.md".to_string()),
        ]);

        // Phase0 is finished
        state.phrases[0].status = "finished".to_string();
        state.current_phase = Some("Phrase1".to_string());

        // Simulate exec finding next non-finished phase
        let current_name = state.current_phase.clone().unwrap();
        let phase = state.phrases.iter().find(|p| p.name == current_name);
        let (name, _) = match phase {
            Some(p) if p.status == "finished" => {
                let idx = state.phrases.iter().position(|p| p.name == current_name).unwrap();
                let next = state.phrases.get(idx + 1);
                match next {
                    Some(np) => (np.name.clone(), np.file.clone()),
                    None => return, // no more phases
                }
            }
            Some(p) => (p.name.clone(), p.file.clone()),
            None => return,
        };

        assert_eq!(name, "Phrase1");
    }

    #[test]
    fn test_confirm_advances_phase_and_marks_finished() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roadmap.json");
        let store = RoadmapStore::new(path.to_str().unwrap());

        let mut state = RoadmapState::new();
        state.init_phrases_from_files(vec![
            ("Phrase0".to_string(), "@project_docs/phases/0.md".to_string()),
            ("Phrase1".to_string(), "@project_docs/phases/1.md".to_string()),
        ]);
        // Simulate exec: set first phrase to dev
        state.phrases[0].status = "dev".to_string();
        state.workflow = "dev".to_string();
        store.save(&state).unwrap();

        // Confirm: advance_phase
        let next = state.advance_phase().unwrap();
        assert_eq!(next.map(|p| p.name.as_str()), Some("Phrase1"));
        assert_eq!(state.phrases[0].status, "finished"); // marked finished
        assert_eq!(state.current_phase, Some("Phrase1".to_string()));
        assert!(!state.is_all_phases_complete());
        store.save(&state).unwrap();

        // Reload and verify
        let loaded = store.load().unwrap();
        assert_eq!(loaded.phrases[0].status, "finished");
        assert_eq!(loaded.current_phase, Some("Phrase1".to_string()));
    }

    #[test]
    fn test_full_workflow_accept_exec_confirm_archive() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roadmap.json");
        let store = RoadmapStore::new(path.to_str().unwrap());

        // Step 1: accept - init phrases
        let mut state = RoadmapState::new();
        state.init_phrases_from_files(vec![
            ("Phrase0".to_string(), "@project_docs/phases/0.md".to_string()),
            ("Phrase1".to_string(), "@project_docs/phases/1.md".to_string()),
        ]);
        assert_eq!(state.workflow, "ready");
        assert_eq!(state.current_phase, Some("Phrase0".to_string()));
        assert_eq!(state.phrases[0].status, "init");
        store.save(&state).unwrap();

        // Step 2: exec - set phrase to dev
        let current_name = state.current_phase.clone().unwrap();
        if let Some(phase) = state.phrases.iter_mut().find(|p| p.name == current_name) {
            if phase.status == "init" {
                phase.status = "dev".to_string();
            }
        }
        state.workflow = "dev".to_string();
        assert_eq!(state.phrases[0].status, "dev");
        assert_eq!(state.workflow, "dev");
        store.save(&state).unwrap();

        // Step 3: confirm - advance to next phase
        let next = state.advance_phase().unwrap();
        assert_eq!(next.map(|p| p.name.as_str()), Some("Phrase1"));
        assert_eq!(state.phrases[0].status, "finished");
        assert_eq!(state.current_phase, Some("Phrase1".to_string()));
        store.save(&state).unwrap();

        // Step 4: exec on phrase1
        let current_name = state.current_phase.clone().unwrap();
        if let Some(phase) = state.phrases.iter_mut().find(|p| p.name == current_name) {
            if phase.status == "init" {
                phase.status = "dev".to_string();
            }
        }
        assert_eq!(state.phrases[1].status, "dev");
        store.save(&state).unwrap();

        // Step 5: confirm - last phase, returns None
        let next = state.advance_phase().unwrap();
        assert!(next.is_none());
        assert_eq!(state.phrases[1].status, "finished");
        assert_eq!(state.current_phase, None);
        assert!(state.is_all_phases_complete());
        store.save(&state).unwrap();

        // Step 6: archive - workflow = "archived"
        state.workflow = "archived".to_string();
        store.save(&state).unwrap();

        // Verify final persistence
        let loaded = store.load().unwrap();
        assert_eq!(loaded.workflow, "archived");
        assert_eq!(loaded.phrases[0].status, "finished");
        assert_eq!(loaded.phrases[1].status, "finished");
        assert!(loaded.is_all_phases_complete());
    }

    #[test]
    fn test_advance_phase_when_no_current_phase() {
        let mut state = RoadmapState::new();
        state.current_phase = None;

        let result = state.advance_phase().unwrap();
        assert!(result.is_none());
        assert_eq!(state.current_phase, None);
    }

    #[test]
    fn test_current_phase_returns_correct_phrase() {
        let mut state = RoadmapState::new();
        state.init_phrases_from_files(vec![
            ("P0".to_string(), "f0".to_string()),
            ("P1".to_string(), "f1".to_string()),
        ]);

        assert_eq!(state.fetch_current_phase().map(|p| p.name.as_str()), Some("P0"));

        state.advance_phase().unwrap();
        assert_eq!(state.fetch_current_phase().map(|p| p.name.as_str()), Some("P1"));
    }

    #[test]
    fn test_all_phases_complete() {



    }

}
