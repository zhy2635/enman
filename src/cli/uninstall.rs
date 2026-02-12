use clap::Args;
use anyhow::Result;
use std::fs;

/// Parses tool@version string into a validated string
fn parse_tool_version(s: &str) -> Result<String, String> {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return Err("Invalid format. Use: tool@version (e.g., 'node@16.14.0')".to_string());
    }
    if parts[0].trim().is_empty() {
        return Err("Tool name cannot be empty".to_string());
    }
    if parts[1].trim().is_empty() {
        return Err("Version cannot be empty".to_string());
    }
    Ok(s.to_string())
}

#[derive(Args)]
pub struct UninstallArgs {
    /// Tool and version to uninstall (e.g., "node@16.14.0")
    #[arg(value_parser = parse_tool_version)]
    pub tool_version: String,
}

pub async fn run(args: UninstallArgs) -> Result<()> {
    let parts: Vec<&str> = args.tool_version.split('@').collect();
    let tool = parts[0];
    let version = parts[1];

    println!("🗑️ Uninstalling {} @ {}", tool, version);
    
    let paths = crate::core::paths::EnvManPaths::new()?;
    let install_path = paths.install_dir(tool).join(version);
    
    if !install_path.exists() {
        println!("⚠️  {} @ {} is not installed", tool, version);
        return Ok(());
    }
    
    // 检查当前全局版本，如果匹配则不允许卸载
    let global_version_file = paths.global_version_file(tool);
    if global_version_file.exists() {
        let current_global = fs::read_to_string(&global_version_file)?.trim().to_string();
        if current_global == version {
            return Err(anyhow::anyhow!(
                "Cannot uninstall {}@{} as it's currently set as the global version. \nPlease switch to another version first with: enman global {}@<other_version>",
                tool, version, tool
            ));
        }
    }
    
    // 删除安装目录
    fs::remove_dir_all(&install_path)?;
    println!("✅ Removed installation directory: {}", install_path.display());
    
    // 如果这是该工具的唯一版本，删除工具目录
    let tool_dir = paths.install_dir(tool);
    if tool_dir.read_dir()?.next().is_none() {
        fs::remove_dir(&tool_dir)?;
        println!("🧹 Removed empty tool directory: {}", tool_dir.display());
    }
    
    // 检查是否有对应的 shim 文件，如果有则提示用户可能需要清理
    let root = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
    let shim_path = if cfg!(windows) {
        root.join(".enman").join("shims").join(format!("{}.exe", tool))
    } else {
        root.join(".enman").join("shims").join(tool)
    };
    
    if shim_path.exists() {
        // 检查是否还有其他版本存在，如果没有，则提示可以删除 shim
        let other_versions_exist = tool_dir.exists() && 
            tool_dir.read_dir().map_or(false, |mut dir| dir.next().is_some());
        
        if !other_versions_exist {
            println!("💡 No more versions of {} installed. You may want to remove the shim at: {}", 
                tool, shim_path.display());
        }
    }
    
    println!("✅ Successfully uninstalled {} {}!", tool, version);
    Ok(())
}