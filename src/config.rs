use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_root_dir")]
    pub root_dir: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
}

fn default_root_dir() -> String {
    ".".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_static_dir() -> String {
    "./static".to_string()
}

impl Config {
    /// 从配置文件加载配置
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 获取根目录的绝对路径
    pub fn get_root_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = fs::canonicalize(&self.root_dir)?;
        Ok(path)
    }

    /// 获取静态文件目录的绝对路径
    pub fn get_static_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = fs::canonicalize(&self.static_dir)?;
        Ok(path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            root_dir: default_root_dir(),
            port: default_port(),
            static_dir: default_static_dir(),
        }
    }
}
