// src/cli/mod.rs
use clap::{Parser, Subcommand};

// 声明所有子模块
pub mod init;
pub mod install;
pub mod global;
pub mod list;
pub mod use_cmd; // 注意：文件名是 use_cmd.rs
pub mod uninstall;
pub mod config;

/// Top-level CLI parser
#[derive(Parser)]
#[command(name = "enman")]
#[command(version)]
#[command(about = crate::tr!("cli_about"), long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Supported subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Install a specific version of a tool
    #[command(alias = "in", aliases = ["i"], about = crate::tr!("command_install_description"))]
    Install(install::InstallArgs),

    /// List installed or available versions
    #[command(alias = "ls", about = crate::tr!("command_list_description"))]
    List(list::ListArgs),

    /// Set the global version for a tool
    #[command(alias = "gl", about = crate::tr!("command_global_description"))]
    Global(global::GlobalArgs),

    /// Use a specific version for the current directory
    #[command(about = crate::tr!("command_use_description"))]
    Use(use_cmd::UseArgs),

    /// Initialize enman configuration
    #[command(about = crate::tr!("command_init_description"))]
    Init(init::InitArgs),

    /// Uninstall a specific version of a tool
    #[command(alias = "un", aliases = ["u"], about = crate::tr!("command_uninstall_description"))]
    Uninstall(uninstall::UninstallArgs),

    /// Manage project configuration
    #[command(about = crate::tr!("command_config_description"))]
    Config(config::ConfigArgs),
}

impl Commands {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Init(args) => init::run(args)?,
            Self::Install(args) => install::run(args).await?,
            Self::Global(args) => global::run(args).await?,
            Self::List(args) => list::run(args).await?,
            Self::Use(args) => use_cmd::run(args).await?,
            Self::Uninstall(args) => uninstall::run(args)?,
            Self::Config(args) => config::run(args)?,
        }
        Ok(())
    }
}

/// Parse tool@version string into (tool, version)
pub fn parse_tool_version(s: &str) -> Result<(String, String), String> {
    let pos = s.find('@')
        .ok_or("Version must be specified with @, e.g., 'node@16.14.0'".to_string())?;

    let tool = s[..pos].to_string();
    let version = s[pos + 1..].to_string();

    if tool.is_empty() {
        return Err("Tool name cannot be empty".to_string());
    }
    if version.is_empty() {
        return Err("Version cannot be empty".to_string());
    }

    Ok((tool, version))
}