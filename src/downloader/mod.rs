// src/downloader/mod.rs
pub mod node;
pub mod java;
pub mod mysql;
pub mod mariadb;  // 添加 MariaDB 模块
pub mod python;  // 添加 Python 模块

use anyhow::Result;
use std::path::Path;

pub async fn install(tool: &str, version: &str, install_dir: &Path) -> Result<()> {
    match tool {
        "node" => node::install(version, install_dir).await,
        "java" | "jdk" => java::install(version, install_dir).await,
        "mysql" => mysql::install(version, install_dir).await,
        "mariadb" => mariadb::install(version, install_dir).await,  // 添加 MariaDB 支持
        "python" => python::install(version, install_dir).await,  // 添加 Python 支持
        _ => Err(anyhow::anyhow!(
            "Unsupported tool: '{}'. Supported: node, java, jdk, mysql, mariadb, python",
            tool
        )),
    }
}

// 获取特定工具的可用版本列表
pub async fn list_available_versions(tool: &str, limit: Option<usize>) -> Result<Vec<String>> {
    match tool {
        "node" => Ok(node::list_available_versions(limit).await?),
        "java" | "jdk" => Ok(java::list_available_versions(limit).await?),
        "python" => Ok(python::list_available_versions(limit).await?),
        "mysql" | "mariadb" => {
            // 这些工具的版本列表功能尚未实现
            let common_versions = vec![
                "8.0".to_string(),
                "5.7".to_string(),
                "5.6".to_string(),
                "5.5".to_string(),
            ];
            let limited_versions = if let Some(l) = limit {
                common_versions.into_iter().take(l).collect()
            } else {
                common_versions
            };
            Ok(limited_versions)
        },
        _ => Err(anyhow::anyhow!(
            "Version listing not supported for tool: '{}'. Supported: node, java, jdk, mysql, mariadb, python",
            tool
        )),
    }
}