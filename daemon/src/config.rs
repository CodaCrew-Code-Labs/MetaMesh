use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub storage_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let storage_path = if cfg!(target_os = "windows") {
            dirs::data_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData")).join("metamesh")
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".metamesh")
        };
        
        Self { storage_path }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }
    
    pub fn save(&self) {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        
        let content = toml::to_string(self).unwrap_or_default();
        fs::write(&config_path, content).ok();
    }
    
    fn config_path() -> PathBuf {
        if cfg!(target_os = "windows") {
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData")).join("metamesh").join("config.toml")
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".metamesh").join("config.toml")
        }
    }
    
    pub fn ensure_storage_dir(&self) {
        fs::create_dir_all(&self.storage_path).ok();
    }
    
    pub fn _private_key_path(&self) -> PathBuf {
        self.storage_path.join("private_key")
    }
}