// src/cli/install.rs
use clap::Args;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Args)]
pub struct InstallArgs {
    /// Tool and version in format: tool@version (e.g., node@20.10.0)
    #[arg(value_parser = parse_tool_version, help = crate::localization::get_localizer().t("arg_tool_version_help"))] 
    pub tool_version: String,
}

fn parse_tool_version(s: &str) -> Result<String, String> {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return Err("Invalid format. Use: tool@version (e.g., 'node@16.14.0')".to_string());
    }
    Ok(s.to_string())
}

fn get_enman_root() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
    Ok(home.join(".enman"))
}

fn create_shim(tool: &str) -> Result<()> {
    let root = get_enman_root()?;
    let shims_dir = root.join("shims");
    std::fs::create_dir_all(&shims_dir)?;

    let shim_path = if cfg!(windows) {
        shims_dir.join(format!("{}.exe", tool))
    } else {
        shims_dir.join(tool)
    };

    if shim_path.exists() {
        return Ok(());
    }

    let current_exe = std::env::current_exe()?;
    
    #[cfg(windows)]
    std::fs::copy(&current_exe, &shim_path)?;
    
    #[cfg(unix)]
    std::os::unix::fs::symlink(&current_exe, &shim_path)?;

    println!("   → Created shim at {}", shim_path.display());
    Ok(())
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let parts: Vec<&str> = args.tool_version.split('@').collect();
    
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid format. Use: tool@version (e.g., 'node@16.14.0')"));
    }
    
    let tool = parts[0];
    let version = parts[1];

    let root = get_enman_root()?;
    let install_dir = root.join("installs");
    let install_path = install_dir.join(tool).join(version);

    if install_path.exists() {
        println!("⚠️  {} @ {} already installed", tool, version);
        return Ok(());
    }

    println!("📦 {} {} @ {}", crate::localization::get_localizer().t("Installing"), tool, version);
    
    // ✅ 核心：委托给 downloader 模块
    crate::downloader::install(tool, version, &install_path).await?;
    
    create_shim(tool)?;
    
    println!("\n✅ Successfully installed {} {}!", tool, version);
    println!("💡 Run `{}` after adding ~/.enman/shims to your PATH", tool);
    Ok(())
}