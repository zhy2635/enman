use crate::core::paths::EnvManPaths;
use anyhow::{anyhow, Result};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tokio;
use std::path::PathBuf;
use zip::ZipArchive;
use flate2::read::GzDecoder;
use tar::Archive;

#[derive(Debug, Deserialize)]
struct RedisRelease {
    tag_name: String,
    assets: Vec<RedisAsset>,
}

#[derive(Debug, Deserialize)]
struct RedisAsset {
    name: String,
    browser_download_url: String,
}

pub async fn install(version: &str, install_dir: &Path) -> Result<()> {
    println!("🔍 Installing redis @ {}", version);

    // 检查是否已安装
    if install_dir.exists() {
        println!("✅ redis @ {} already installed", version);
        println!("💡 Tip: to reinstall please uninstall first");
        return Ok(());
    }

    // 确保安装目录存在
    fs::create_dir_all(install_dir)?;

    // 创建临时目录用于下载
    let temp_dir = std::env::temp_dir().join(format!("enman_redis_temp_{}", version));
    fs::create_dir_all(&temp_dir)?;
    
    let archive_path = download_redis(&temp_dir, version).await?;
    extract_archive(&archive_path, install_dir)?;
    
    // 清理临时目录
    fs::remove_dir_all(&temp_dir)?;

    println!("🎉 Successfully installed redis @ {}", version);
    Ok(())
}

pub async fn install_redis_version(version: &str) -> Result<()> {
    let paths = EnvManPaths::new()?;
    let install_dir = paths.install_dir("redis").join(version);
    install(version, &install_dir).await
}

async fn download_redis(temp_dir: &Path, version: &str) -> Result<PathBuf> {
    let client = create_http_client();

    // 构建下载URL
    let download_url = if cfg!(windows) {
        // Windows上的Redis通常使用TPoradowski的分发
        // 由于官方不提供Windows版本，我们使用GitHub上的第三方构建
        // 首先尝试最可能存在的URL格式
        let urls_to_try = [
            format!("https://github.com/tporadowski/redis/releases/download/v{}/Redis-{}-x64.zip", version, version),
            format!("https://github.com/tporadowski/redis/releases/download/{}/Redis-{}-x64.zip", version, version),
            format!("https://github.com/redis-windows/redis/releases/download/{}/redis-{}.zip", version, version),
        ];

        let mut last_error = None;
        for url in &urls_to_try {
            println!("📥 Downloading redis @ {} from: {}", version, url);

            match client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let archive_path = temp_dir.join(format!("redis-{}{}", version, ".zip"));

                        let bytes = response.bytes().await?;
                        fs::write(&archive_path, bytes)?;

                        println!("✅ Download completed from: {}", url);
                        return Ok(archive_path);
                    } else {
                        last_error = Some(anyhow!("HTTP Error: {}", response.status()));
                        println!("⚠️ Download failed from: {} - HTTP Error: {}", url, response.status());
                        continue;
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow!("Network Error: {}", e));
                    println!("⚠️ Network error when downloading from: {} - {}", url, e);
                    continue;
                }
            }
        }

        // 如果所有Windows URL都失败了，抛出错误
        return Err(match last_error {
            Some(e) => e,
            None => anyhow!("No Windows Redis download URLs attempted"),
        });
    } else {
        // Linux/Mac版本 - 从Redis官网获取
        format!("https://download.redis.io/releases/redis-{}.tar.gz", version)
    };

    println!("📥 Downloading redis @ {}...", version);
    println!("🌐 {}", download_url);

    let response: reqwest::Response = client
        .get(&download_url)
        .send()
        .await?;

    if response.status().is_client_error() || response.status().is_server_error() {
        return Err(anyhow!(
            "Failed to download Redis {}. HTTP Error: {}",
            version,
            response.status()
        ));
    }

    let file_extension = if cfg!(windows) { ".zip" } else { ".tar.gz" };
    let archive_path = temp_dir.join(format!("redis-{}{}", version, file_extension));

    let bytes = response.bytes().await?;
    fs::write(&archive_path, bytes)?;

    println!("✅ Download completed");

    Ok(archive_path)
}

fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    if archive_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        // 处理ZIP文件
        let file = std::fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(dest_dir)?;
    } else if archive_path.extension().and_then(|s| s.to_str()) == Some("gz") {
        // 处理tar.gz文件
        let file = std::fs::File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.unpack(dest_dir)?;
    } else {
        return Err(anyhow!("Unsupported archive format"));
    }

    Ok(())
}

fn create_http_client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

pub async fn list_redis_versions() -> Result<Vec<String>> {
    // 对于Windows，从tporadowski/redis获取版本
    // 对于Linux/Mac，我们暂时返回一些常见版本
    if cfg!(windows) {
        let client = create_http_client();
        let url = "https://api.github.com/repos/tporadowski/redis/releases";

        let response: reqwest::Response = client
            .get(url)
            .header("User-Agent", "enman")
            .send()
            .await?;

        if !response.status().is_success() {
            // 如果API调用失败，返回一些常见版本
            return Ok(vec!["7.2.4".to_string(), "7.0.5".to_string(), "6.2.7".to_string()]);
        }

        let releases: Vec<RedisRelease> = response.json().await?;
        let mut versions: Vec<String> = releases
            .iter()
            .filter_map(|release| {
                release.tag_name.strip_prefix('v').map(|v| v.to_string())
            })
            .collect();

        // 去重并排序
        versions.sort_by(|a, b| crate::core::version::compare_versions(b, a));
        versions.dedup();

        Ok(versions)
    } else {
        // 对于非Windows系统，返回通用版本列表
        Ok(vec!["7.4.2".to_string(), "7.2.4".to_string(), "7.0.5".to_string(), "6.2.7".to_string()])
    }
}

pub fn is_redis_installed(version: &str) -> bool {
    let paths = match EnvManPaths::new() {
        Ok(paths) => paths,
        Err(_) => return false,
    };

    let install_dir = paths.install_dir("redis").join(version);
    install_dir.exists()
}

pub fn get_redis_install_path(version: &str) -> Result<std::path::PathBuf> {
    let paths = EnvManPaths::new()?;
    Ok(paths.install_bin_path("redis", version))
}

pub fn uninstall_redis_version(version: &str) -> Result<()> {
    let paths = EnvManPaths::new()?;
    let install_dir = paths.install_dir("redis").join(version);

    if !install_dir.exists() {
        return Err(anyhow!("redis @ {} is not installed", version));
    }

    // 检查是否为全局版本
    let global_version_file = paths.global_version_file("redis");
    if global_version_file.exists() {
        let global_version = fs::read_to_string(&global_version_file)?.trim().to_string();
        if global_version == version {
            return Err(anyhow!(
                "Cannot uninstall redis @ {} as it is set as global version. Run `enman global redis@<other_version>` or `enman global --unset redis` first.",
                version
            ));
        }
    }

    fs::remove_dir_all(&install_dir)?;
    println!("🗑️ Uninstalled redis @ {}", version);

    // 检查是否还有其他Redis版本
    let redis_dir = paths.install_dir("redis");
    if redis_dir.exists() && is_redis_dir_empty(&redis_dir)? {
        fs::remove_dir_all(&redis_dir)?;
        println!("🧹 Removed empty redis directory");
    }

    Ok(())
}

fn is_redis_dir_empty(redis_dir: &Path) -> Result<bool> {
    if !redis_dir.exists() {
        return Ok(true);
    }

    let entries: Result<Vec<_>, _> = fs::read_dir(redis_dir)?.collect();
    Ok(entries?.is_empty())
}