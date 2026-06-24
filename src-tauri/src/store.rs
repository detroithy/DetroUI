use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

pub struct Store {
    data: Value,
    path: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("detbar")
            .join("config.json");

        let data = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| Self::defaults())
        } else {
            Self::defaults()
        };

        let store = Store { data, path };
        store.save();
        store
    }

    fn defaults() -> Value {
        json!({
            "theme": "dark",
            "accentColor": "#00d4ff",
            "taskbarPosition": "bottom",
            "taskbarHeight": 48,
            "widgetPositions": {},
            "enabledWidgets": ["clock", "system-monitor"],
            "launcherPinnedApps": [],
            "weatherCity": "Istanbul",
            "wallpaperPath": "",
            "wallpaperVolume": 0,
            "opacity": 88,
            "autoAssignWindows": true,
            "appDesktopMap": {}
        })
    }

    pub fn get(&self, key: &str) -> Value {
        self.data.get(key).cloned().unwrap_or(Value::Null)
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.data[key] = value;
        self.save();
    }

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.path, json);
        }
    }
}
