// src/downloader/node.rs
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::fs;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;

// 注意：这些函数现在只服务于 node，所以可以简化

fn detect_platform() -> Result<(&'static str, &'static str, &'static str)> {
    let os = if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err(anyhow::anyhow!("Unsupported OS"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err(anyhow::anyhow!("Unsupported architecture"));
    };

    let ext = if cfg!(windows) {
        "zip"
    } else if cfg!(target_os = "linux") {
        "tar.xz"
    } else {
        "tar.gz"
    };

    Ok((os, arch, ext))
}

fn get_download_urls(version: &str) -> Result<Vec<String>> {
    let (os, arch, ext) = detect_platform()?;
    let filename = format!("node-v{}-{}-{}.{}", version, os, arch, ext);
    
    Ok(vec![
        format!("https://nodejs.org/dist/v{}/{}", version, filename),
        format!("https://npmmirror.com/mirrors/node/v{}/{}", version, filename),
    ])
}

async fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    let res = reqwest::get(url).await.context("Failed to start download")?;
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("HTTP {}: {}", res.status(), url));
    }

    let total = res
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("Missing content-length"))?;
    
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("█░"));
    
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

fn extract_and_flatten(archive: &Path, dest: &Path, format: &str) -> Result<()> {
    fs::create_dir_all(dest)?;
    let temp_extract = TempDir::new()?;

    eprintln!("🔧 Extracting as {}...", format);

    match format {
        "zip" => {
            let file = fs::File::open(archive)?;
            let mut zip = zip::ZipArchive::new(file)?;
            zip.extract(temp_extract.path())?;
        }
        "tar.xz" => {
            let file = fs::File::open(archive)?;
            let decoder = xz2::read::XzDecoder::new(file);
            let mut tar = tar::Archive::new(decoder);
            tar.unpack(temp_extract.path())?;
        }
        "tar.gz" => {
            let file = fs::File::open(archive)?;
            let decoder = flate2::read::GzDecoder::new(file);
            let mut tar = tar::Archive::new(decoder);
            tar.unpack(temp_extract.path())?;
        }
        _ => return Err(anyhow::anyhow!("Unsupported archive format: {}", format)),
    }

    // Flatten: move contents of top-level dir to dest
    let entries: Vec<_> = fs::read_dir(temp_extract.path())?
        .collect::<std::io::Result<Vec<_>>>()?;

    if entries.len() != 1 || !entries[0].file_type()?.is_dir() {
        return Err(anyhow::anyhow!("Expected a single top-level directory in archive"));
    }

    let inner_dir = entries[0].path();
    for entry in fs::read_dir(inner_dir)? {
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
        for bin_name in ["node", "npm", "npx"] {
            let bin_path = dest.join("bin").join(bin_name);
            if bin_path.exists() {
                let mut perms = fs::metadata(&bin_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&bin_path, perms)?;
            }
        }
    }

    Ok(())
}

pub async fn install(version: &str, install_dir: &Path) -> Result<()> {
    let node_bin = if cfg!(windows) {
        install_dir.join("node.exe")
    } else {
        install_dir.join("bin").join("node")
    };

    if node_bin.exists() {
        println!("⚠️  Node.js {} already installed", version);
        return Ok(());
    }

    let (_os, _arch, ext) = detect_platform()?;
    let urls = get_download_urls(version)?;
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join(format!("node.{}", ext));

    let mut success = false;
    for (i, url) in urls.iter().enumerate() {
        let source = if i == 0 { "Official" } else { "Mirror (npmmirror)" };
        eprintln!("📥 [{}] Trying: {}", source, url);
        
        if download_with_progress(url, &archive_path).await.is_ok() {
            eprintln!("✅ Using source: {}", source);
            success = true;
            break;
        } else {
            eprintln!("⚠️  [{}] Failed", source);
        }
    }

    if !success {
        return Err(anyhow::anyhow!("All download sources failed"));
    }

    extract_and_flatten(&archive_path, install_dir, ext)
        .context("Failed to extract Node.js")?;

    if !node_bin.exists() {
        return Err(anyhow::anyhow!("Verification failed: node binary not found at {:?}", node_bin));
    }

    eprintln!("✨ Node.js {} installed to {}", version, install_dir.display());
    Ok(())
}

// 获取 Node.js 可用版本的函数
pub async fn list_available_versions(limit: Option<usize>) -> Result<Vec<String>> {
    // 从 Node.js API 获取版本列表
    let url = "https://nodejs.org/dist/index.json";
    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch Node.js versions list"));
    }
    
    let json: serde_json::Value = response.json().await?;
    
    if let Some(versions_array) = json.as_array() {
        let mut versions: Vec<String> = versions_array
            .iter()
            .filter_map(|item| item.get("version").and_then(|v| v.as_str()))
            .map(|v| v.strip_prefix('v').unwrap_or(v).to_string())
            .collect();
        
        // 按版本号排序
        versions.sort_by(|a, b| version_compare(a, b));
        
        if let Some(limit) = limit {
            versions.truncate(limit);
        }
        
        Ok(versions)
    } else {
        Err(anyhow::anyhow!("Invalid response format from Node.js API"))
    }
}

// 辅助函数：比较版本号
fn version_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<u32> = a.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let b_parts: Vec<u32> = b.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    
    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let a_val = if i < a_parts.len() { a_parts[i] } else { 0 };
        let b_val = if i < b_parts.len() { b_parts[i] } else { 0 };
        
        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    
    std::cmp::Ordering::Equal
}