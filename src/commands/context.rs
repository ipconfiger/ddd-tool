use crate::state::RoadmapStore;
use crate::state::RoadmapState;
use anyhow::Result;
use std::path::PathBuf;

pub struct DddContext {
    pub store: RoadmapStore,
    pub project_root: PathBuf,
}

impl DddContext {
    pub fn new() -> Result<Self> {
        let project_root = std::env::current_dir()?;
        let roadmap_path = project_root.join("project_docs").join("roadmap.json");

        let store = RoadmapStore::new(roadmap_path.to_str().unwrap());

        Ok(Self {
            store,
            project_root,
        })
    }

    pub fn load_state(&self) -> Result<RoadmapState> {
        self.store.load()
    }

    pub fn save_state(&self, state: &RoadmapState) -> Result<()> {
        self.store.save(state)
    }

    pub fn resolve_path(&self, path: &str) -> PathBuf {
        let path = path.trim_start_matches("@project_docs/");
        self.project_root.join("project_docs").join(path)
    }
}
