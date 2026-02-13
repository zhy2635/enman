use crate::core::paths;
use anyhow::{bail, Result};
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct UninstallArgs {
    /// Tool and version to uninstall (e.g., "node@16.14.0")
    #[arg(value_parser = crate::cli::parse_tool_version)]
    pub tool: (String, String),
}

pub fn run(args: UninstallArgs) -> Result<()> {
    let (tool, version) = args.tool;
    
    // 特定工具的卸载逻辑
    match tool.as_str() {
        "redis" => {
            return crate::downloader::redis::uninstall_redis_version(&version);
        }
        // 检查特定工具模块中是否有对应的卸载函数
        // 如果没有，使用通用逻辑
        _ => {}
    }
    
    let env_paths = paths::EnvManPaths::new()?;
    let install_dir = env_paths.install_dir(&tool);
    let install_path = install_dir.join(&version);
    
    // 检查版本是否已安装
    if !install_path.exists() {
        println!("{} @ {} is not installed", tool, version);
        return Ok(());
    }

    // 检查是否是全局版本
    let global_version_file = env_paths.global_version_file(&tool);
    if global_version_file.exists() {
        if let Ok(global_content) = fs::read_to_string(&global_version_file) {
            let global_version = global_content.trim();
            
            // 检查全局版本是否为 "tool@version" 格式或仅为版本号
            let is_global = if global_version.contains('@') {
                global_version == format!("{}@{}", tool, version)
            } else {
                global_version == version
            };
            
            if is_global {
                println!("Cannot uninstall {} @ {} as it is set as global", tool, version);
                println!("Run `enman global {}@<other_version>` to switch first", tool);
                return Ok(());
            }
        }
    }

    // 删除安装目录
    fs::remove_dir_all(&install_path)?;
    println!("Removed installation directory: {}", install_path.display());

    // 检查是否还有其他版本
    let versions: Vec<_> = install_dir
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter(|entry| entry.file_name() != ".locks")
        .collect();

    if versions.is_empty() {
        println!("No more versions of {} installed. You may want to remove the shim at: {}/shims/{}.exe", 
            tool, 
            env_paths.root.display(),
            tool
        );
    }

    println!("Uninstalled {} {}!", tool, version);

    Ok(())
}