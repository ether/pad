pub mod paths;

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub remote: Option<String>,
    #[serde(default)]
    pub consented_remotes: Vec<String>,
    #[serde(default)]
    pub telemetry: bool,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::config_root().join("config.json");
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Config::default());
            }
            Err(e) => return Err(anyhow::anyhow!("read config: {e}")),
        };
        let cfg: Config =
            serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("parse config: {e}"))?;
        Ok(cfg)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = paths::config_root();
        fs::create_dir_all(&dir)?;
        let raw = serde_json::to_string_pretty(self)?;
        fs::write(dir.join("config.json"), raw)?;
        Ok(())
    }
}
