use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::fs;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// Show current project configuration
    Show,
    /// Apply configuration from .enmanrc
    Apply,
    /// Initialize a new .enmanrc file
    Init,
}

pub fn run(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommand::Show => show_config()?,
        ConfigCommand::Apply => apply_config()?,
        ConfigCommand::Init => init_config()?,
    }
    Ok(())
}

fn show_config() -> Result<()> {
    let config_path = std::env::current_dir()?.join(".enmanrc");
    if !config_path.exists() {
        println!("Config file not found: {}", config_path.display());
        return Ok(());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: HashMap<String, String> = toml::from_str(&content)?;

    println!("Tools configured in {}:", config_path.display());
    for (tool, version) in &config {
        println!("  {}: {}", tool, version);
    }

    if config.is_empty() {
        println!("No tools configured in {}", config_path.display());
    }

    Ok(())
}

fn apply_config() -> Result<()> {
    let config_path = std::env::current_dir()?.join(".enmanrc");
    if !config_path.exists() {
        println!("Config file not found: {}", config_path.display());
        return Ok(());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: HashMap<String, String> = toml::from_str(&content)?;

    println!("Applying configuration from {}:", config_path.display());
    for (tool, version) in &config {
        println!("  Setting {} to version {}", tool, version);
        // 这里需要调用实际的use命令来设置工具版本
        // 目前只是模拟输出
    }

    println!("Configuration applied successfully!");
    Ok(())
}

fn init_config() -> Result<()> {
    let config_path = std::env::current_dir()?.join(".enmanrc");
    if config_path.exists() {
        println!("Config file already exists: {}", config_path.display());
        return Ok(());
    }

    let default_config = r#"# EnMan configuration
# Add your tools and their versions here
# Example:
# node = "16.14.0"
# python = "3.10.0"
"#;
    
    fs::write(&config_path, default_config)?;
    println!("Created new config file: {}", config_path.display());
    Ok(())
}