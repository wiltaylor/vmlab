//! Registry of lab daemons (name → root, pid, state), persisted to the
//! state dir so a restarted supervisor can re-adopt running labs.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabState {
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabEntry {
    pub name: String,
    pub root: PathBuf,
    pub pid: u32,
    pub state: LabState,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    labs: Vec<LabEntry>,
}

impl Registry {
    fn path() -> PathBuf {
        crate::paths::state_dir().join("labs.json")
    }

    pub fn load() -> Self {
        match std::fs::read_to_string(Self::path()) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, s).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    pub fn labs(&self) -> &[LabEntry] {
        &self.labs
    }

    pub fn get(&self, name: &str) -> Option<&LabEntry> {
        self.labs.iter().find(|l| l.name == name)
    }

    pub fn upsert(&mut self, entry: LabEntry) {
        match self.labs.iter_mut().find(|l| l.name == entry.name) {
            Some(slot) => *slot = entry,
            None => self.labs.push(entry),
        }
    }

    pub fn set_state(&mut self, name: &str, state: LabState) {
        if let Some(l) = self.labs.iter_mut().find(|l| l.name == name) {
            l.state = state;
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.labs.retain(|l| l.name != name);
    }
}
