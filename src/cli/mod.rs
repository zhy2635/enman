// src/cli/mod.rs
use clap::Subcommand; // ← 必须导入 Subcommand 和 Args

// 声明所有子模块
pub mod init;
pub mod install;
pub mod global;
pub mod list;
pub mod use_cmd; // 注意：文件名是 use_cmd.rs
pub mod uninstall; // 新增 uninstall 模块
pub mod config; // 新增 config 模块

// ✅ 关键：把 parse_tool_version 设为 pub
pub fn parse_tool_version(s: &str) -> Result<(String, String), &'static str> {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Expected format: tool@version (e.g., node@20.10.0)");
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

// ✅ 正确使用 #[derive(Subcommand)]
#[derive(Subcommand)]
pub enum Commands {
    #[command(about = crate::tr!("command_init_description"))]
    Init(init::InitArgs),
    #[command(alias = "in", aliases = ["i"], about = crate::tr!("command_install_description"))]
    Install(install::InstallArgs),
    #[command(about = crate::tr!("command_global_description"))]
    Global(global::GlobalArgs),
    #[command(alias = "ls", about = crate::tr!("command_list_description"))]
    List(list::ListArgs),
    #[command(about = crate::tr!("command_use_description"))]
    Use(use_cmd::UseArgs),
    #[command(alias = "un", aliases = ["u"], about = crate::tr!("command_uninstall_description"))]
    Uninstall(uninstall::UninstallArgs),
    #[command(about = crate::tr!("command_config_description"))]
    Config(config::ConfigArgs),
}

impl Commands {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Init(args) => init::run(args).await,
            Self::Install(args) => install::run(args).await,
            Self::Global(args) => global::run(args).await,
            Self::List(args) => list::run(args).await,
            Self::Use(args) => use_cmd::run(args).await, // ← 直接调用模块函数
            Self::Uninstall(args) => uninstall::run(args).await, // 添加 uninstall 命令处理
            Self::Config(args) => config::run(args).await, // 添加 config 命令处理
        }
    }
}