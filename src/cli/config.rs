use clap::Args;
use anyhow::Result;
use std::fs;
use std::path::Path;
use toml;

#[derive(Args)]
pub struct ConfigArgs {
    /// Action to perform: show, apply, init
    #[arg(value_name = "ACTION")]
    pub action: String,

    /// Optional config file path
    #[arg(short, long, default_value = "./.enmanrc")]
    pub file: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct EnmanConfig {
    #[serde(default)]
    tools: std::collections::HashMap<String, String>,
}

pub async fn run(args: ConfigArgs) -> Result<()> {
    match args.action.as_str() {
        "show" => show_config(&args.file)?,
        "apply" => apply_config(&args.file).await?,
        "init" => init_config(&args.file)?,
        _ => return Err(anyhow::anyhow!("Unknown action: {}. Available: show, apply, init", args.action)),
    }
    Ok(())
}

fn show_config(config_path: &str) -> Result<()> {
    if !Path::new(config_path).exists() {
        println!("Config file not found: {}", config_path);
        return Ok(());
    }

    let content = fs::read_to_string(config_path)?;
    let config: EnmanConfig = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

    println!("Tools configured in {}:", config_path);
    for (tool, version) in &config.tools {
        println!("  {}: {}", tool, version);
    }

    Ok(())
}

async fn apply_config(config_path: &str) -> Result<()> {
    if !Path::new(config_path).exists() {
        return Err(anyhow::anyhow!("Config file does not exist: {}", config_path));
    }

    let content = fs::read_to_string(config_path)?;
    let config: EnmanConfig = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

    if config.tools.is_empty() {
        println!("No tools configured in {}", config_path);
        return Ok(());
    }

    println!("Applying configuration from {}:", config_path);
    for (tool, version) in &config.tools {
        println!("  Setting {} to version {}", tool, version);
        
        // 使用 enman use 命令来设置本地版本
        let tool_version = (tool.clone(), version.clone());
        let args = crate::cli::use_cmd::UseArgs { tool: tool_version };
        // 直接调用 use_cmd 模块的函数，不再使用 block_on
        crate::cli::use_cmd::run(args).await?;
    }

    println!("✅ Configuration applied successfully!");

    Ok(())
}

fn init_config(config_path: &str) -> Result<()> {
    if Path::new(config_path).exists() {
        return Err(anyhow::anyhow!("Config file already exists: {}", config_path));
    }

    let default_config = EnmanConfig {
        tools: std::collections::HashMap::new(),
    };

    let content = toml::to_string(&default_config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize default config: {}", e))?;

    fs::write(config_path, content)?;
    println!("✅ Created new config file: {}", config_path);

    Ok(())
}