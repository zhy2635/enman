// src/cli/install.rs
use crate::core::paths;
use crate::downloader;
use anyhow::Result;
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct InstallArgs {
    /// Tool and version to install (e.g., "node@16.14.0")
    #[arg(value_parser = crate::cli::parse_tool_version)]
    pub tool: (String, String),
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let (tool, version) = args.tool;
    
    let env_paths = paths::EnvManPaths::new()?;
    let install_dir = env_paths.install_dir(&tool);
    let install_path = install_dir.join(&version);
    
    // 检查是否已安装
    if install_path.exists() {
        println!("{} @ {} already installed", tool, version);
        return Ok(());
    }
    
    println!("Installing {} {}", tool, version);

    // 安装工具
    downloader::install(&tool.to_lowercase(), &version, &install_path).await?;

    // 创建shim
    let shims_dir = env_paths.root.join("shims");
    fs::create_dir_all(&shims_dir)?;
    
    let shim_exe = shims_dir.join(format!("{}.exe", tool));
    // 复制当前可执行文件到shim位置
    let current_exe = std::env::current_exe()?;
    fs::copy(current_exe, &shim_exe)?;

    println!("Installed {} {} successfully!", tool, version);

    Ok(())
}