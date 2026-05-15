use std::path::PathBuf;

pub fn state_root() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| dirs::home_dir().expect("HOME").join(".local/state"))
        .join("pad")
}

pub fn config_root() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().expect("HOME").join(".config"))
        .join("pad")
}
