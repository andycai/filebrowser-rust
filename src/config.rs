use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "rootDirs", default = "default_root_dirs")]
    pub root_dirs: Vec<RootDirConfig>,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(rename = "staticDirs", default = "default_static_dirs")]
    pub static_dirs: Vec<StaticDirConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootDirConfig {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticDirConfig {
    pub name: String,
    pub path: String,
}

fn default_root_dirs() -> Vec<RootDirConfig> {
    vec![RootDirConfig {
        name: "默认目录".to_string(),
        path: ".".to_string(),
    }]
}

fn default_port() -> u16 {
    8080
}

fn default_static_dirs() -> Vec<StaticDirConfig> {
    vec![StaticDirConfig {
        name: "default".to_string(),
        path: "./static".to_string(),
    }]
}

impl Config {
    /// 从配置文件加载配置
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            root_dirs: default_root_dirs(),
            port: default_port(),
            static_dirs: default_static_dirs(),
        }
    }
}
