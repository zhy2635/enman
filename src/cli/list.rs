// src/cli/list.rs
use clap::Args;
use anyhow::Result;
use std::fs;
use crate::core::paths::EnvManPaths;

#[derive(Args)]
pub struct ListArgs {
    /// Optional: tool name (e.g., 'node'). If omitted, show current global versions of all tools.
    #[arg(value_name = "TOOL", help = crate::localization::get_localizer().t("arg_tool_help"))]
    pub tool: Option<String>,

    /// List all available tools that can be downloaded
    #[arg(long = "available", short = 'a', conflicts_with = "tool", help = crate::localization::get_localizer().t("arg_available_help"))]
    pub available: bool,
    
    /// List remote versions available for download
    #[arg(long = "remote", short = 'r', requires = "tool", help = crate::localization::get_localizer().t("arg_remote_help"))]
    pub remote: bool,

    /// Show system information like TLS version, OS info, etc
    #[arg(long = "sys-info", short = 's', help = "Show system information like TLS version, OS info, etc")]
    pub sys_info: bool,
}

pub async fn run(args: ListArgs) -> Result<()> {
    if args.sys_info {
        show_system_info()?;
    }

    let paths = EnvManPaths::new()?;

    if args.available {
        list_available_tools().await
    } else if args.remote {
        if let Some(ref tool) = args.tool {
            list_remote_versions(tool).await
        } else {
            // 这种情况不应该发生，因为 --remote 需要 --tool
            list_current_global_versions(&paths)
        }
    } else {
        match args.tool {
            Some(tool) => list_tool_versions_detailed(&paths, &tool)?,
            None => list_current_global_versions(&paths)?,
        }
        Ok(())
    }
}

// === 显示系统信息 ===
fn show_system_info() -> Result<()> {
    use std::process::Command;
    use std::env;
    
    println!("=== System Information ===");
    
    // 显示操作系统信息
    println!("OS: {} {}", env::consts::OS, env::consts::ARCH);
    
    // 显示Rust TLS后端信息
    println!("TLS Backend: {}",
        if cfg!(feature = "native-tls") { "native-tls" }
        else if cfg!(feature = "rustls") { "rustls" }
        else { "default" });
    
    // 尝试检测OpenSSL版本（如果可用）
    #[cfg(target_family = "unix")]
    {
        if let Ok(output) = Command::new("openssl").arg("version").output() {
            if !output.stdout.is_empty() {
                let openssl_version = String::from_utf8_lossy(&output.stdout);
                println!("OpenSSL: {}", openssl_version.trim());
            }
        }
    }
    
    // 显示enman版本
    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        println!("enman version: {}", version);
    }
    
    // 显示Rust版本
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if !output.stdout.is_empty() {
            let rust_version = String::from_utf8_lossy(&output.stdout);
            println!("Rust: {}", rust_version.trim());
        }
    }
    
    // 显示当前时间
    println!("Current Time: {:?}", chrono::Utc::now());
    
    println!("========================");
    Ok(())
}

// === 列出特定工具的远程可用版本 ===
async fn list_remote_versions(tool: &str) -> Result<()> {
    match crate::downloader::list_available_versions(tool, Some(10)).await {
        Ok(versions) => {
            println!("{} {}:", crate::localization::get_localizer().t("Latest versions"), tool);
            for version in versions {
                println!("  {}", version);
            }
        }
        Err(e) => {
            eprintln!("{} {} {}: {}", crate::localization::get_localizer().t("Failed to fetch"), tool, crate::localization::get_localizer().t("versions"), e);
            
            // 根据工具类型提供一些常见的版本作为示例
            match tool {
                "node" => {
                    println!("{}:", crate::localization::get_localizer().t("Common Node.js versions"));
                    println!("  20.x.x (LTS)");
                    println!("  18.x.x (LTS)");
                    println!("  16.x.x (LTS)");
                    println!("  14.x.x (LTS)");
                }
                "java" => {
                    println!("{}:", crate::localization::get_localizer().t("Common Java versions"));
                    println!("  21 (LTS)");
                    println!("  17 (LTS)");
                    println!("  11 (LTS)");
                    println!("  8 (Legacy LTS)");
                }
                "python" => {
                    println!("{}:", crate::localization::get_localizer().t("Common Python versions"));
                    println!("  3.12.x (Latest)");
                    println!("  3.11.x");
                    println!("  3.10.x");
                    println!("  3.9.x");
                }
                "mysql" | "mariadb" => {
                    println!("{} {} {}:", crate::localization::get_localizer().t("Supported"), tool, crate::localization::get_localizer().t("versions"));
                    println!("  8.0.x");
                    println!("  5.7.x");
                }
                _ => {
                    eprintln!("{} '{}' {}", crate::localization::get_localizer().t("Tool is not supported for remote version listing"), tool, "");
                }
            }
        }
    }
    Ok(())
}

// === 列出所有可下载的工具 ===
async fn list_available_tools() -> Result<()> {
    println!("{}", crate::localization::get_localizer().t("Available tools that can be installed:"));
    println!("  node    - Node.js JavaScript runtime");
    println!("  java    - OpenJDK Java Development Kit");
    println!("  jdk     - OpenJDK Java Development Kit (alias for java)");
    println!("  mysql   - MySQL database server");
    println!("  mariadb - MariaDB database server (community fork of MySQL)");
    println!("  python  - Python programming language");

    println!("\n{}:", crate::localization::get_localizer().t("To see available versions for a tool, use"));
    println!("  enman list <tool> --remote");
    
    Ok(())
}

// === 详细模式：列出某个工具的所有已安装版本 ===
fn list_tool_versions_detailed(paths: &EnvManPaths, tool: &str) -> Result<()> {
    let install_dir = paths.install_dir(tool);
    if !install_dir.exists() {
        println!("{} {} {}.", crate::localization::get_localizer().t("No versions of"), tool, crate::localization::get_localizer().t("installed"));
        return Ok(());
    }

    let global_version = fs::read_to_string(paths.global.join(tool))
        .ok()
        .map(|s| s.trim().to_string());

    // 检查本地版本（当前目录下的 .enman-version 文件）
    let local_version = std::env::current_dir()
        .ok()
        .and_then(|dir| {
            let local_file = dir.join(".enman-version");
            if local_file.exists() {
                fs::read_to_string(local_file).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .and_then(|content| {
            // 解析本地版本文件内容，格式为 "tool@version"
            let parts: Vec<&str> = content.split('@').collect();
            if parts.len() == 2 && parts[0] == tool {
                Some(parts[1].to_string())
            } else {
                None
            }
        });

    let mut versions: Vec<String> = fs::read_dir(install_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.file_name().to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect();

    if versions.is_empty() {
        println!("{} {} {}.", crate::localization::get_localizer().t("No versions of"), tool, crate::localization::get_localizer().t("installed"));
        return Ok(());
    }

    versions.sort();

    for version in &versions {
        if local_version.as_ref() == Some(version) {
            println!("->{} ({})", version, crate::localization::get_localizer().t("local"));
        } else if global_version.as_ref() == Some(version) {
            println!("->{} ({})", version, crate::localization::get_localizer().t("global"));
        } else {
            println!("  {}", version);
        }
    }

    // 如果本地版本不同于全局版本且不在安装列表中，也显示它
    if let Some(local_ver) = &local_version {
        if global_version.as_ref() != Some(local_ver) && !versions.contains(local_ver) {
            println!("->{} ({}, {})", local_ver, crate::localization::get_localizer().t("local"), crate::localization::get_localizer().t("not installed"));
        }
    }

    Ok(())
}

// === 摘要模式：列出所有工具的当前全局版本 ===
fn list_current_global_versions(paths: &EnvManPaths) -> Result<()> {
    if !paths.global.exists() {
        println!("{}", crate::localization::get_localizer().t("No global versions set."));
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&paths.global)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
            } else {
                None
            }
        })
        .collect();

    if entries.is_empty() {
        println!("{}", crate::localization::get_localizer().t("No global versions set."));
        return Ok(());
    }

    entries.sort();

    for tool in entries {
        if let Ok(version) = fs::read_to_string(paths.global.join(&tool)) {
            let version = version.trim();
            if !version.is_empty() {
                // 检查当前目录是否有本地版本覆盖
                if let Ok(current_dir) = std::env::current_dir() {
                    let local_file = current_dir.join(".enman-version");
                    if local_file.exists() {
                        if let Ok(local_content) = fs::read_to_string(&local_file) {
                            let local_parts: Vec<&str> = local_content.trim().split('@').collect();
                            if local_parts.len() == 2 && local_parts[0] == tool {
                                println!("{}: {} ({}: {})", tool, version, crate::localization::get_localizer().t("local"), local_parts[1]);
                                continue;
                            }
                        }
                    }
                }
                
                println!("{}: {}", tool, version);
            }
        }
    }

    Ok(())
}