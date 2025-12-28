use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "rootDirs", default = "default_root_dirs")]
    pub root_dirs: Vec<RootDirConfig>,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(rename = "staticDir", default = "default_static_dir")]
    pub static_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootDirConfig {
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

    /// 获取静态文件目录的绝对路径
    pub fn get_static_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = fs::canonicalize(&self.static_dir)?;
        Ok(path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            root_dirs: default_root_dirs(),
            port: default_port(),
            static_dir: default_static_dir(),
        }
    }
}
