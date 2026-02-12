use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::fs;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;

fn detect_platform() -> Result<(&'static str, &'static str, &'static str)> {
    let os = if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err(anyhow::anyhow!("Unsupported OS"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err(anyhow::anyhow!("Unsupported architecture"));
    };

    let ext = if cfg!(windows) {
        "zip"  // 使用 ZIP 格式，因为我们要下载便携版本而不是安装程序
    } else {
        "tgz"  // 使用 tgz 作为 tar.gz 的表示
    };

    Ok((os, arch, ext))
}

fn get_download_urls(version: &str) -> Result<Vec<String>> {
    let (os, arch, _ext) = detect_platform()?;
    
    let urls = match os {
        "win" => {
            // 对于 Windows，使用嵌入式 Python 版本，这是一个便携版本，无需安装
            let filename = match arch {
                "x86_64" => format!("python-{}-embed-amd64.zip", version),
                "aarch64" => format!("python-{}-embed-arm64.zip", version),
                _ => format!("python-{}-embed-amd64.zip", version), // 默认回退
            };
            
            vec![
                format!("https://www.python.org/ftp/python/{}/{}", version, filename),
                // 提供备用镜像
                format!("https://npm.taobao.org/mirrors/python/{}/{}", version, filename),
            ]
        },
        "macos" => {
            // 对于 macOS，使用 python-build-standalone 提供的便携版本
            let suffix = match arch {
                "x86_64" => "x86_64-apple-darwin-install_only.tar.gz",
                "aarch64" => "aarch64-apple-darwin-install_only.tar.gz",
                _ => "x86_64-apple-darwin-install_only.tar.gz", // 默认回退
            };
            
            vec![
                format!("https://github.com/indygreg/python-build-standalone/releases/download/20231002/cpython-{}+20231002-{}", 
                        version, suffix),
            ]
        },
        "linux" => {
            // 对于 Linux，同样使用 python-build-standalone 提供的便携版本
            let suffix = match arch {
                "x86_64" => "x86_64-unknown-linux-gnu-install_only.tar.gz",
                "aarch64" => "aarch64-unknown-linux-gnu-install_only.tar.gz",
                _ => "x86_64-unknown-linux-gnu-install_only.tar.gz", // 默认回退
            };
            
            vec![
                format!("https://github.com/indygreg/python-build-standalone/releases/download/20231002/cpython-{}+20231002-{}", 
                        version, suffix),
            ]
        },
        _ => return Err(anyhow::anyhow!("Unsupported OS")),
    };
    
    Ok(urls)
}

async fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (compatible; enman)")  // 设置浏览器兼容的 User-Agent
        .build()?;

    let res = client.get(url).send().await.context("Failed to start download")?;
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

fn extract_and_install_python(archive: &Path, dest: &Path, ext: &str) -> Result<()> {
    fs::create_dir_all(dest)?;
    
    let temp_extract = TempDir::new()?;
    
    eprintln!("🔧 Extracting as {}...", ext);

    match ext {
        "tgz" | "tar.gz" => {
            let file = fs::File::open(archive)?;
            let decoder = flate2::read::GzDecoder::new(file);
            let mut tar = tar::Archive::new(decoder);
            
            // 特殊处理 Python 构建，解压到子目录后再移动
            tar.unpack(temp_extract.path())?;
            
            // 获取临时目录中的所有文件
            for entry_result in fs::read_dir(temp_extract.path())? {
                let entry = entry_result.map_err(|e| anyhow::anyhow!("Error processing entry: {}", e))?;
                
                let target = dest.join(entry.file_name());
                
                if target.exists() {
                    fs::remove_dir_all(&target).ok();
                }
                
                fs::rename(entry.path(), &target)?;
            }
        },
        "zip" => {
            let file = fs::File::open(archive)?;
            let mut zip = zip::ZipArchive::new(file)?;
            
            // 解压 ZIP 文件
            zip.extract(temp_extract.path())?;
            
            // 获取临时目录中的所有文件
            for entry_result in fs::read_dir(temp_extract.path())? {
                let entry = entry_result.map_err(|e| anyhow::anyhow!("Error processing entry: {}", e))?;
                
                let target = dest.join(entry.file_name());
                
                if target.exists() {
                    fs::remove_dir_all(&target).ok();
                }
                
                // 移动文件到目标目录
                fs::rename(entry.path(), &target)?;
            }
        },
        _ => return Err(anyhow::anyhow!("Unsupported archive format: {}", ext)),
    }

    // 确保 Python 可执行文件存在并设置权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let python_bin = dest.join("python");
        if python_bin.exists() {
            let mut perms = fs::metadata(&python_bin)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&python_bin, perms)?;
        }
        
        // 尝试设置 python3 权限
        let python3_bin = dest.join("python3");
        if python3_bin.exists() {
            let mut perms = fs::metadata(&python3_bin)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&python3_bin, perms)?;
        }
    }

    // 对于 Windows，确保 python.exe 存在
    #[cfg(windows)]
    {
        let python_exe = dest.join("python.exe");
        if !python_exe.exists() {
            // 尝试查找以 python 开头的 exe 文件
            for entry_result in fs::read_dir(dest)? {
                let entry = match entry_result {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                
                if entry.file_type()?.is_file() {
                    let file_name = entry.file_name();
                    if let Some(name) = file_name.to_str() {
                        if name.starts_with("python") && name.ends_with(".exe") {
                            // 重命名为标准的 python.exe
                            fs::rename(dest.join(name), dest.join("python.exe"))?;
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn install(version: &str, install_dir: &Path) -> Result<()> {
    let python_bin = if cfg!(windows) {
        install_dir.join("python.exe")
    } else {
        install_dir.join("python")
    };

    if python_bin.exists() {
        println!("⚠️  Python {} already installed", version);
        return Ok(());
    }

    let (_os, _arch, ext) = detect_platform()?;
    let urls = get_download_urls(version)?;
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join(format!("python.{}", ext));

    let mut success = false;
    for (i, url) in urls.iter().enumerate() {
        let source = if i == 0 { 
            if cfg!(windows) { "Official Python Embedded" } else { "Python Build Standalone" } 
        } else { 
            "Mirror" 
        };
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
        // 提供手动安装说明
        eprintln!("❌ Unable to automatically download Python.");
        eprintln!("");
        eprintln!("💡 Manual installation steps:");
        eprintln!("   1. Visit: https://www.python.org/downloads/");
        eprintln!("   2. For Windows: Download 'Embeddable zip file' for your version");
        eprintln!("   3. For Unix: Consider using python-build-standalone releases");
        eprintln!("   4. Extract to: {}", install_dir.display());
        eprintln!("   5. Run: enman global python@{}", version);
        return Err(anyhow::anyhow!("Automatic download failed."));
    }

    extract_and_install_python(&archive_path, install_dir, &ext)
        .context("Failed to extract and install Python")?;

    if !python_bin.exists() {
        // 检查是否有其他可能的 Python 可执行文件
        let mut found_python = false;
        for entry_result in fs::read_dir(install_dir)? {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            if entry.file_type()?.is_file() {
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str() {
                    if (cfg!(windows) && name.starts_with("python") && name.ends_with(".exe")) ||
                       (cfg!(unix) && name == "python") {
                        found_python = true;
                        break;
                    }
                }
            }
        }
        
        if !found_python {
            return Err(anyhow::anyhow!("Verification failed: python executable not found at {:?}", python_bin));
        }
    }

    println!("✨ Python {} installed to {}", version, install_dir.display());

    Ok(())
}

// 获取 Python 可用版本的函数
pub async fn list_available_versions(limit: Option<usize>) -> Result<Vec<String>> {
    // 从 PyPI API 获取最新版本
    let url = "https://pypi.org/pypi/python/json";
    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch Python versions list"));
    }
    
    let json: serde_json::Value = response.json().await?;
    
    if let Some(releases) = json.get("releases").and_then(|v| v.as_object()) {
        let mut versions: Vec<String> = releases.keys().cloned().collect();
        
        // 按版本号排序
        versions.sort_by(|a, b| version_compare(a, b));
        versions.reverse(); // 从新到旧
        
        if let Some(limit) = limit {
            versions.truncate(limit);
        }
        
        Ok(versions)
    } else {
        Err(anyhow::anyhow!("Invalid response format from PyPI API"))
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