use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirstRunState {
    pub has_seen_onboarding: bool,
    pub skipped: bool,
    pub completed: bool,
    pub last_started_unix_ms: Option<i64>,
    pub last_finished_unix_ms: Option<i64>,
}

impl Default for FirstRunState {
    fn default() -> Self {
        Self {
            has_seen_onboarding: false,
            skipped: false,
            completed: false,
            last_started_unix_ms: None,
            last_finished_unix_ms: None,
        }
    }
}

pub trait KvStore {
    fn get_string(&self, key: &str) -> Option<String>;
    fn set_string(&mut self, key: &str, value: String) -> Result<(), String>;
}

pub struct FileKvStore {
    path: PathBuf,
}

impl FileKvStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn read_all(&self) -> serde_json::Map<String, serde_json::Value> {
        let s = match fs::read_to_string(&self.path) {
            Ok(x) => x,
            Err(_) => return serde_json::Map::new(),
        };
        let v: serde_json::Value = match serde_json::from_str(&s) {
            Ok(x) => x,
            Err(_) => return serde_json::Map::new(),
        };
        match v {
            serde_json::Value::Object(m) => m,
            _ => serde_json::Map::new(),
        }
    }

    fn write_all(&self, m: serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create_dir_all failed: {}", e))?;
        }
        let v = serde_json::Value::Object(m);
        let s =
            serde_json::to_string_pretty(&v).map_err(|e| format!("json encode failed: {}", e))?;
        fs::write(&self.path, s).map_err(|e| format!("write failed: {}", e))?;
        Ok(())
    }
}

impl KvStore for FileKvStore {
    fn get_string(&self, key: &str) -> Option<String> {
        let m = self.read_all();
        m.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    fn set_string(&mut self, key: &str, value: String) -> Result<(), String> {
        let mut m = self.read_all();
        m.insert(key.to_string(), serde_json::Value::String(value));
        self.write_all(m)
    }
}

pub struct FirstRun {
    store: Box<dyn KvStore>,
    key: String,
}

impl FirstRun {
    pub fn new(store: Box<dyn KvStore>) -> Self {
        Self {
            store,
            key: "onboarding.first_run_state.v1".to_string(),
        }
    }

    pub fn load(&self) -> FirstRunState {
        let Some(s) = self.store.get_string(&self.key) else {
            return FirstRunState::default();
        };
        serde_json::from_str(&s).unwrap_or_else(|_| FirstRunState::default())
    }

    pub fn save(&mut self, st: &FirstRunState) -> Result<(), String> {
        let s =
            serde_json::to_string(st).map_err(|e| format!("encode FirstRunState failed: {}", e))?;
        self.store.set_string(&self.key, s)
    }

    pub fn should_auto_start(&self) -> bool {
        let st = self.load();
        !st.has_seen_onboarding && !st.skipped && !st.completed
    }

    pub fn mark_started(&mut self, now_unix_ms: i64) -> Result<(), String> {
        let mut st = self.load();
        st.has_seen_onboarding = true;
        st.last_started_unix_ms = Some(now_unix_ms);
        self.save(&st)
    }

    pub fn mark_skipped(&mut self) -> Result<(), String> {
        let mut st = self.load();
        st.skipped = true;
        self.save(&st)
    }

    pub fn mark_completed(&mut self, now_unix_ms: i64) -> Result<(), String> {
        let mut st = self.load();
        st.completed = true;
        st.last_finished_unix_ms = Some(now_unix_ms);
        self.save(&st)
    }

    pub fn reset_for_rerun(&mut self) -> Result<(), String> {
        let st = FirstRunState::default();
        self.save(&st)
    }
}

pub fn default_first_run_store_path() -> PathBuf {
    if let Ok(p) = std::env::var("CRAFTCAD_USER_DATA_DIR") {
        return PathBuf::from(p).join("onboarding.json");
    }
    PathBuf::from("user_data").join("onboarding.json")
}
