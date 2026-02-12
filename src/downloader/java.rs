// src/downloader/java.rs
#[allow(dead_code)]
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::fs;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;

fn detect_platform() -> Result<(&'static str, &'static str)> {
    let os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err(anyhow::anyhow!("Unsupported OS"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported architecture. Supported: x64, aarch64"
        ));
    };

    Ok((os, arch))
}


async fn get_download_url(version: &str, os: &str, arch: &str) -> Result<String> {
    // Step 1: 从 Adoptium API 获取最新 release 的文件名
    let metadata_url = format!(
        "https://api.adoptium.net/v3/assets/feature_releases/{}/ga?architecture={}&os={}&image_type=jdk&archive_type=zip&sort_method=DEFAULT&sort_order=DESC&vendor=eclipse",
        version, arch, os
    );

    eprintln!("   → Fetching release metadata from: {}", metadata_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .context("Failed to build HTTP client")?;

    let res = client.get(&metadata_url).send().await.context("API fetch failed")?;
    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Adoptium API error: {}", body));
    }

    let json: serde_json::Value = res.json().await.context("JSON parse failed")?;
    let releases = json.as_array().ok_or_else(|| anyhow::anyhow!("Expected JSON array"))?;

    if releases.is_empty() {
        return Err(anyhow::anyhow!("No JDK found for Java {} on {} {}", version, os, arch));
    }

    // 提取文件名（不含路径）
    let filename = releases[0]["binaries"][0]["package"]["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'package.name' not found"))?;

    // Step 2: 构造 TUNA 镜像 URL（使用标准目录结构）
    // TUNA 路径: /17/jdk/x64/windows/filename.zip
    let tuna_url = format!(
        "https://mirrors.tuna.tsinghua.edu.cn/Adoptium/{}/{}/{}/{}/{}",
        version,
        "jdk",
        arch,
        os,
        filename
    );

    eprintln!("   → Using TUNA mirror: {}", tuna_url);
    Ok(tuna_url)
}
async fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    eprintln!("   → Downloading (with retry)...");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5分钟超时
        .connect_timeout(std::time::Duration::from_secs(30))
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .context("Failed to build download client")?;

    let mut attempt = 0;
    const MAX_RETRIES: u32 = 3;

    loop {
        attempt += 1;
        match download_once(&client, url, dest).await {
            Ok(()) => return Ok(()),
            Err(e) if attempt < MAX_RETRIES => {
                eprintln!("   ⚠️  Attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                eprintln!("   → Retrying in 2 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn download_once(client: &reqwest::Client, url: &str, dest: &Path) -> Result<()> {
    let res = client
        .get(url)
        .send()
        .await
        .context("Failed to start download")?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("HTTP {}: {}", res.status(), url));
    }

    let total = res
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("Missing content-length"))?;

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("█░"),
    );

    let mut file = tokio::fs::File::create(dest).await?;
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
        let chunk = chunk.context("Download error")?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }
    pb.finish_with_message("✓ Downloaded");
    Ok(())
}

fn extract_and_flatten(archive: &Path, dest: &Path, _is_zip: bool) -> Result<()> {
    fs::create_dir_all(dest)?;
    let temp_extract = TempDir::new()?;

    eprintln!("   → Extracting JDK...");

    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    zip.extract(temp_extract.path())?;

    let entries: Vec<_> = fs::read_dir(temp_extract.path())?
        .collect::<std::io::Result<Vec<_>>>()?;

    let jdk_root = entries
        .into_iter()
        .find(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .ok_or_else(|| anyhow::anyhow!("No top-level directory found in archive"))?;

    for entry in fs::read_dir(jdk_root.path())? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if target.exists() {
            fs::remove_dir_all(&target).ok();
        }
        fs::rename(entry.path(), &target)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let java_bin = dest.join("bin").join("java");
        if java_bin.exists() {
            let mut perms = fs::metadata(&java_bin)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&java_bin, perms)?;
        }
    }

    Ok(())
}

pub async fn install(version: &str, install_dir: &Path) -> Result<()> {
    let java_bin = if cfg!(windows) {
        install_dir.join("bin").join("java.exe")
    } else {
        install_dir.join("bin").join("java")
    };

    if java_bin.exists() {
        println!("⚠️  Java {} already installed at {}", version, install_dir.display());
        return Ok(());
    }

    let (os, arch) = detect_platform()?;

    eprintln!("🔍 Fetching download URL for Java {} ({}, {})...", version, os, arch);
    let download_url = get_download_url(version, os, arch).await?;

    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("jdk.zip");

    download_with_progress(&download_url, &archive_path).await
        .context("Failed to download JDK")?;

    extract_and_flatten(&archive_path, install_dir, true)
        .context("Failed to extract JDK")?;

    if !java_bin.exists() {
        return Err(anyhow::anyhow!(
            "Verification failed: java binary not found at {:?}",
            java_bin
        ));
    }

    eprintln!("✨ Java {} installed successfully to {}", version, install_dir.display());
    Ok(())
}

// 获取 Java 可用版本的函数
pub async fn list_available_versions(limit: Option<usize>) -> Result<Vec<String>> {
    // 从 Adoptium API 获取所有可用的 Java 版本
    let url = "https://api.adoptium.net/v3/info/available_releases";
    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch Java versions list"));
    }
    
    let json: serde_json::Value = response.json().await?;
    
    // 提取所有特性版本
    if let Some(feature_versions) = json.get("available_releases").and_then(|v| v.as_array()) {
        let mut versions: Vec<String> = feature_versions
            .iter()
            .filter_map(|v| v.as_i64())
            .map(|v| v.to_string())
            .collect();
        
        // 按数字大小降序排列（最新版本在前）
        versions.sort_by(|a, b| b.cmp(a));
        
        if let Some(limit) = limit {
            versions.truncate(limit);
        }
        
        Ok(versions)
    } else {
        Err(anyhow::anyhow!("Invalid response format from Adoptium API"))
    }
}