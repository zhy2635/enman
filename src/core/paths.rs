// src/core/paths.rs
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EnvManPaths {
    #[allow(dead_code)] 
    pub root: PathBuf,
    pub shims: PathBuf,
    pub installs: PathBuf,
    pub global: PathBuf,
}

impl EnvManPaths {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        let root = home.join(".enman");
        Ok(Self {
            shims: root.join("shims"),
            installs: root.join("installs"),
            global: root.join("global"),
            root,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.shims)?;
        std::fs::create_dir_all(&self.installs)?;
        std::fs::create_dir_all(&self.global)?;
        Ok(())
    }

    pub fn global_version_file(&self, tool: &str) -> PathBuf {
        self.global.join(tool)
    }

    /// 返回工具二进制所在的目录（不是完整路径，不含 exe 名）
    pub fn install_bin_path(&self, tool: &str, version: &str) -> PathBuf {
        let base = self.installs.join(tool).join(version);
        if tool == "node" {
            if cfg!(windows) {
                base // Windows: node.exe is at top level
            } else {
                base.join("bin") // Unix: node is in bin/
            }
        } else if tool == "java" {
            // Java always puts binaries in bin/ (on all platforms)
            base.join("bin")
        } else if tool == "python" {
            // Python embedded version puts python.exe at top level (on all platforms)
            base
        } else {
            // Default assumption: most tools put binaries in bin/
            base.join("bin")
        }
    }

    pub fn install_dir(&self, tool: &str) -> PathBuf {
        self.installs.join(tool)
    }
}