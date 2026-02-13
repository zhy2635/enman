use crate::core::paths;
use crate::downloader;
use anyhow::Result;
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct UseArgs {
    /// Tool and version to set locally (e.g., "node@16.14.0")
    #[arg(value_parser = crate::cli::parse_tool_version)]
    pub tool: (String, String),
}

pub async fn run(args: UseArgs) -> Result<()> {
    let (tool, version) = args.tool;
    
    let env_paths = paths::EnvManPaths::new()?;
    let install_dir = env_paths.install_dir(&tool);
    let install_path = install_dir.join(&version);
    
    // 检查版本是否已安装
    if !install_path.exists() {
        println!("Installing {} @ {}", tool, version);
        downloader::install(&tool.to_lowercase(), &version, &install_path).await?;
    } else {
        println!("Switching to {} @ {}", tool, version);
    }

    // 如果是MySQL，停止当前服务并启动新服务
    if tool.to_lowercase() == "mysql" {
        let local_version_file = std::env::current_dir()?.join(".enman-version");
        let mut should_start_new_service = true;
        
        if local_version_file.exists() {
            if let Ok(current_use_content) = fs::read_to_string(&local_version_file) {
                let current_parts: Vec<&str> = current_use_content.trim().split('@').collect();
                if current_parts.len() == 2 {
                    let current_tool = current_parts[0];
                    let current_version = current_parts[1];
                    
                    if current_tool == tool && current_version != version {
                        // 使用MySQL模块的函数停止当前运行的服务
                        if let Err(e) = crate::downloader::mysql::stop_current_mysql_service() {
                            eprintln!("Could not stop current MySQL service: {}", e);
                        } else {
                            println!("Stopped previous MySQL service");
                        }
                    } else if current_tool != tool {
                        // 如果当前本地版本是另一个工具，也需要先停止当前服务
                        if let Err(e) = crate::downloader::mysql::stop_current_mysql_service() {
                            eprintln!("Could not stop current MySQL service: {}", e);
                        } else {
                            println!("Stopped previous MySQL service");
                        }
                    }
                }
            }
        } else {
            // 如果当前没有设置此工具的本地版本，则只需启动新服务
            if let Err(e) = crate::downloader::mysql::start_mysql_service(&install_path) {
                eprintln!("Could not start new MySQL service: {}", e);
            } else {
                println!("Started MySQL service for version {}", version);
            }
            should_start_new_service = false; // 我们已经启动了服务，不需要再次启动
        }
        
        // 启动新版本的服务（除非我们已经启动过了）
        if should_start_new_service {
            if let Err(e) = crate::downloader::mysql::start_mysql_service(&install_path) {
                eprintln!("Could not start new MySQL service: {}", e);
            } else {
                println!("Started new MySQL service for version {}", version);
            }
        }
    }

    // 设置为本地版本
    let local_version_file = std::env::current_dir()?.join(".enman-version");
    fs::write(&local_version_file, format!("{}@{}", tool, version))?;  // 保存 tool@version 格式，因为需要知道工具名称
    println!("Set local {} to version {}", tool, version);

    Ok(())
}