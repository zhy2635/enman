// src/em.rs
// em - enman 的别名入口点
// 这个文件提供与 enman 相同的功能，但使用较短的命令名

mod localization; // 本地化模块必须首先定义
mod cli;
mod core;
mod downloader;
use clap::Parser;
use std::env;
use std::path::PathBuf;

// ====== 查找本地 .enmanrc 版本（支持 TOML 格式） ======
fn find_local_version(tool: &str, start_dir: PathBuf) -> Option<String> {
    let mut current = start_dir;
    loop {
        let enmanrc = current.join(".enmanrc");
        if enmanrc.exists() {
            if let Ok(content) = std::fs::read_to_string(&enmanrc) {
                // 首先尝试解析为 TOML 格式
                if let Ok(toml_config) = toml::from_str::<std::collections::HashMap<String, toml::Value>>(&content) {
                    if let Some(tools_obj) = toml_config.get("tools") {
                        if let Some(tools) = tools_obj.as_table() {
                            if let Some(version_value) = tools.get(tool) {
                                if let Some(version_str) = version_value.as_str() {
                                    let version_clean = version_str.trim_matches('"');
                                    return Some(version_clean.to_string());
                                }
                            }
                        }
                    }
                } else {
                    // 如果 TOML 解析失败，回退到旧的 key=value 解析方式
                    for line in content.lines() {
                        let line = line.trim();
                        // 跳过空行和注释
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        // 解析 key=value
                        if let Some((key, value)) = line.split_once('=') {
                            if key.trim() == tool {
                                let version = value.trim();
                                let version_clean = version.trim_matches('"');
                                if !version_clean.is_empty() {
                                    return Some(version_clean.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 到达根目录则停止
        if !current.pop() {
            break;
        }
    }
    None
}

// ====== Shim 转发逻辑（同步） ======
fn get_tool_name_from_exe() -> Option<String> {
    let exe_path = env::current_exe().ok()?;
    let exe_name = exe_path.file_stem()?.to_str()?;
    // 如果是 em，则继续作为主程序运行
    // 如果是 shim 工具（如 node.exe, python.exe），则转发到对应工具
    if exe_name == "em" { None } else { Some(exe_name.to_string()) }
}

fn run_tool(tool: &str, args: &[String]) -> anyhow::Result<()> {
    eprintln!("[DEBUG] Current working directory: {}", env::current_dir()?.display());
    eprintln!("[SHIM] Detected tool: '{}'", tool);

    let paths = crate::core::paths::EnvManPaths::new()?;

    // 🔍 1. 尝试从 .enmanrc 获取本地版本
    let local_version = find_local_version(tool, env::current_dir()?);

    let version = if let Some(v) = local_version {
        eprintln!("[LOCAL] Using {}@{} from .enmanrc", tool, v);
        v
    } else {
        // 🌐 2. 回退到全局版本
        let version_file = paths.global_version_file(tool);
        if !version_file.exists() {
            eprintln!("Error: no global version set for '{}'.", tool);
            eprintln!("Run: em global {}@<version>", tool);
            std::process::exit(1);
        }

        let version = std::fs::read_to_string(&version_file)?
            .trim()
            .to_string();

        if version.is_empty() {
            eprintln!("Error: global version file for '{}' is empty", tool);
            std::process::exit(1);
        }
        version
    };

    // ✅ 构建二进制路径
    let bin_dir = paths.install_bin_path(tool, &version);
    eprintln!("[DEBUG] install_bin_path('{}', '{}') = {}", tool, version, bin_dir.display());

    let bin_name = if cfg!(windows) {
        format!("{}.exe", tool)
    } else {
        tool.to_string()
    };
    let tool_bin = bin_dir.join(bin_name);
    eprintln!("[DEBUG] Final binary path: {}", tool_bin.display());

    if !tool_bin.exists() {
        eprintln!("Error: {}@{} is not installed (looked for {})", tool, version, tool_bin.display());
        std::process::exit(1);
    }

    // 🚀 执行工具
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = std::process::Command::new(&tool_bin).args(args).exec();
        eprintln!("Failed to execute {}: {}", tool, error);
        std::process::exit(1);
    }

    #[cfg(windows)]
    {
        let status = std::process::Command::new(&tool_bin)
            .args(args)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", tool, e))?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

// ====== CLI 入口 ======
#[derive(Parser)]
#[command(name = "em", version, about = crate::localization::get_localizer().t("app_description"))]
struct EmApp {
    #[command(subcommand)]
    command: cli::Commands,
}

impl EmApp {
    async fn run(self) -> anyhow::Result<()> {
        self.command.execute().await
    }
}

// ====== 主函数 ======
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(tool) = get_tool_name_from_exe() {
        let args = env::args().skip(1).collect::<Vec<String>>();
        run_tool(&tool, &args)?;
        Ok(())
    } else {
        let app = EmApp::parse();
        app.run().await
    }
}