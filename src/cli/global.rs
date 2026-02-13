use crate::core::paths;
use crate::downloader;
use anyhow::Result;
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct GlobalArgs {
    /// Tool and version to set globally (e.g., "node@16.14.0")
    #[arg(value_parser = crate::cli::parse_tool_version)]
    pub tool: (String, String),
}

pub async fn run(args: GlobalArgs) -> Result<()> {
    let (tool, version) = args.tool;
    
    let env_paths = paths::EnvManPaths::new()?;
    let install_dir = env_paths.install_dir(&tool);
    let install_path = install_dir.join(&version);
    
    // 检查版本是否已安装
    if !install_path.exists() {
        println!("Installing {} @ {}", tool, version);
        downloader::install(&tool.to_lowercase(), &version, &install_path).await?;
    } else {
        println!("Setting {} @ {} as global", tool, version);
    }

    // 如果是MySQL，停止当前服务并启动新服务
    if tool.to_lowercase() == "mysql" {
        let global_version_file = env_paths.global_version_file(&tool);
        if global_version_file.exists() {
            if let Ok(current_global_content) = fs::read_to_string(&global_version_file) {
                let current_parts: Vec<&str> = current_global_content.trim().split('@').collect();
                if current_parts.len() == 2 {
                    let current_tool = current_parts[0];
                    let current_version = current_parts[1];
                    
                    if current_tool == tool && current_version != version {
                        // 停止当前运行的服务
                        if let Err(e) = crate::downloader::mysql::stop_current_mysql_service() {
                            eprintln!("Could not stop current MySQL service: {}", e);
                        } else {
                            println!("Stopped previous MySQL service");
                        }
                    }
                }
            }
        }
        
        // 启动新版本的服务
        if let Err(e) = crate::downloader::mysql::start_mysql_service(&install_path) {
            eprintln!("Could not start new MySQL service: {}", e);
        } else {
            println!("Started new MySQL service for version {}", version);
        }
    }

    // 设置为全局版本
    let global_version_file = env_paths.global_version_file(&tool);
    fs::write(&global_version_file, &version)?;  // 只保存版本号，而不是 tool@version 格式
    println!("Set global {} to version {}", tool, version);

    Ok(())
}