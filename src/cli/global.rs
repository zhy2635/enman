// src/cli/global.rs
use clap::Args;
use crate::core::paths;
use crate::downloader;
use anyhow::Result;

#[derive(Args)]
pub struct GlobalArgs {
    /// Tool and version to set globally (e.g., "node@16.14.0")
    #[arg(help = crate::localization::get_localizer().t("arg_tool_version_help"))]
    pub tool: String,
}

pub async fn run(args: GlobalArgs) -> Result<()> {
    let tool_version = args.tool;
    let parts: Vec<&str> = tool_version.split('@').collect();
    
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid format. Use: tool@version (e.g., 'node@16.14.0')"));
    }
    
    let tool = parts[0];
    let version = parts[1];
    
    let env_paths = paths::EnvManPaths::new()?;
    let install_dir = env_paths.install_dir(tool);
    let install_path = install_dir.join(version);
    
    // 检查版本是否已安装
    if !install_path.exists() {
        println!("📦 {} {} @ {}", crate::localization::get_localizer().t("Installing"), tool, version);
        downloader::install(&tool.to_lowercase(), version, &install_path).await?;
    } else {
        println!("🔄 {} {} @ {}", crate::localization::get_localizer().t("Switching to"), tool, version);
    }

    // 如果是MySQL，停止当前服务并启动新服务
    if tool.to_lowercase() == "mysql" {
        let global_version_file = env_paths.global_version_file(tool);
        
        if global_version_file.exists() {
            if let Ok(current_global_version) = std::fs::read_to_string(&global_version_file) {
                if current_global_version.trim() != version {
                    // 使用MySQL模块的函数停止当前运行的服务
                    if let Err(e) = crate::downloader::mysql::stop_current_mysql_service() {
                        eprintln!("⚠️ {} {}", crate::localization::get_localizer().t("Warning: Could not stop current MySQL service:"), e);
                    } else {
                        println!("✅ {}", crate::localization::get_localizer().t("Stopped previous MySQL service"));
                    }
                    
                    // 启动新版本的服务
                    if let Err(e) = crate::downloader::mysql::start_mysql_service(&install_path) {
                        eprintln!("⚠️ {} {}", crate::localization::get_localizer().t("Warning: Could not start new MySQL service:"), e);
                        eprintln!("💡 {}", crate::localization::get_localizer().t("Please start MySQL manually using: .\\bin\\mysqld --defaults-file=my.ini --console"));
                    } else {
                        println!("✅ {} {}", crate::localization::get_localizer().t("Started new MySQL service for version"), version);
                    }
                }
            }
        }
    }

    // 设置为全局版本
    std::fs::write(env_paths.global_version_file(tool), version)?;
    println!("✅ {} {} {} {}", crate::localization::get_localizer().t("Successfully set global"), tool, crate::localization::get_localizer().t("to version"), version);

    Ok(())
}