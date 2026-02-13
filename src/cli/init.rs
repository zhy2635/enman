use crate::core::paths;
use anyhow::Result;
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct InitArgs {}

pub fn run(_args: InitArgs) -> Result<()> {
    let env_paths = paths::EnvManPaths::new()?;
    let config_path = env_paths.root.join("enman.json");
    
    if config_path.exists() {
        println!("Config file already exists: {}", config_path.display());
        println!("If you want to recreate it, please remove the existing file first.");
        return Ok(());
    }

    // 创建基本配置文件
    fs::create_dir_all(&env_paths.root)?;
    fs::write(&config_path, "{}")?;

    println!("Created new config file: {}", config_path.display());
    println!("You can now add tools and their versions to manage per project");

    Ok(())
}