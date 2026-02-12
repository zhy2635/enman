// src/downloader/mariadb.rs
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::fs;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use std::process::Command;

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
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err(anyhow::anyhow!("Unsupported architecture"));
    };

    let ext = if cfg!(windows) {
        "zip"
    } else {
        "tar.gz"  // Using tar.gz for Unix systems
    };

    Ok((os, arch, ext))
}

fn get_download_urls(version: &str) -> Result<Vec<String>> {
    let (os, arch, _) = detect_platform()?;
    
    // Create filename based on OS - corrected for actual MariaDB naming
    let filename = if os == "win" {
        format!("mariadb-{}-{}-{}.{}", version, "winx64", "debugsymbols", "zip")  // Updated format based on observed URL
    } else if os == "macos" {
        format!("mariadb-{}-{}.{}", version, "macos-x86_64", "tar.gz")
    } else {
        format!("mariadb-{}-{}-{}.{}", version, os, arch, "tar.gz")
    };
    
    // Determine the platform string for the URL
    let platform_str = if os == "win" { 
        "winx64-packages".to_string()  // Convert to string to match return type
    } else if os == "macos" { 
        "x86_64".to_string() 
    } else { 
        format!("{}-{}", os, arch) 
    };
    
    // MariaDB is open-source and freely downloadable
    let mut urls = Vec::new();
    
    if os == "win" {
        // Specific URLs for Windows - using the correct path structure
        urls.extend(vec![
            format!("https://mirrors.tuna.tsinghua.edu.cn/mariadb/mariadb-{}/{}/{}", version, platform_str, filename),
            format!("https://archive.mariadb.org/mariadb-{}/{}", version, filename),
            format!("https://ftp.nluug.nl/db/mariadb/mariadb-{}/{}/{}", version, platform_str, filename),
        ]);
    } else {
        // URLs for Unix-like systems
        urls.extend(vec![
            format!("https://mirrors.tuna.tsinghua.edu.cn/mariadb/mariadb-{}/bintar-{}/{}", 
                    version, platform_str, filename),
            format!("https://archive.mariadb.org/mariadb-{}/bintar-{}/{}", 
                    version, platform_str, filename),
            format!("https://ftp.nluug.nl/db/mariadb/mariadb-{}/bintar-{}/{}", 
                    version, platform_str, filename),
        ]);
    }
    
    Ok(urls)
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

    eprintln!("🔧 Extracting MariaDB as {}...", format);

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

    // Set executable permissions for Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        // Make MariaDB executables executable
        let bin_dir = dest.join("bin");
        if bin_dir.exists() {
            for bin_name in ["mysqld", "mysql", "mysqladmin", "mysqldump", "mysqlcheck"] {
                let bin_path = bin_dir.join(bin_name);
                if bin_path.exists() {
                    let mut perms = fs::metadata(&bin_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&bin_path, perms)?;
                }
            }
        }
    }

    Ok(())
}

pub async fn install(version: &str, install_dir: &Path) -> Result<()> {
    // Check if MariaDB is already installed
    let mariadb_bin = if cfg!(windows) {
        install_dir.join("bin").join("mysqld.exe")
    } else {
        install_dir.join("bin").join("mysqld")
    };

    if mariadb_bin.exists() {
        println!("⚠️  MariaDB {} already installed", version);
        return Ok(());
    }

    let (_os, _arch, ext) = detect_platform()?;
    let urls = get_download_urls(version)?;
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join(format!("mariadb.{}", ext));

    let mut success = false;
    for (i, url) in urls.iter().enumerate() {
        let source = match i {
            0 => "Tsinghua Mirror",
            1 => "MariaDB Archive",
            2 => "NLUUG FTP",
            _ => "Other Mirror"
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
        eprintln!("❌ Unable to automatically download MariaDB.");
        eprintln!("");
        eprintln!("💡 Manual installation steps:");
        eprintln!("   1. Visit: https://mariadb.org/download/");
        eprintln!("   2. Download MariaDB Server for your OS");
        eprintln!("   3. Extract to: {}", install_dir.display());
        eprintln!("   4. Run: enman global mariadb@{}", version);
        eprintln!("");
        eprintln!("🔄 Alternatively, try installing a different version:");
        eprintln!("   enman install mariadb@11.4.2");
        eprintln!("");
        eprintln!("📋 MariaDB is fully compatible with MySQL commands and syntax.");
        eprintln!("");
        return Err(anyhow::anyhow!("Automatic download failed. See instructions above."));
    }

    extract_and_flatten(&archive_path, install_dir, ext)
        .context("Failed to extract MariaDB")?;

    if !mariadb_bin.exists() {
        return Err(anyhow::anyhow!("Verification failed: mysqld binary not found at {:?}", mariadb_bin));
    }

    eprintln!("✨ MariaDB {} installed to {}", version, install_dir.display());
    
    // Initialize MariaDB data directory and get temporary root password
    initialize_mariadb(install_dir, version).await?;
    
    Ok(())
}

async fn initialize_mariadb(install_dir: &Path, _version: &str) -> Result<()> {
    eprintln!("🔐 Initializing MariaDB data directory...");
    
    let data_dir = install_dir.join("data");
    fs::create_dir_all(&data_dir)?;
    
    // Determine binary name based on platform
    let mysqld_bin = if cfg!(windows) {
        install_dir.join("bin").join("mysqld.exe")
    } else {
        install_dir.join("bin").join("mysqld")
    };
    
    // Run mysqld --initialize to set up the data directory
    let output = if cfg!(windows) {
        Command::new(&mysqld_bin)
            .arg("--initialize")
            .arg(format!("--datadir={}", data_dir.display()))
            .arg(format!("--basedir={}", install_dir.display()))
            .output()?
    } else {
        // On Unix-like systems, we may need to set the HOME directory
        Command::new(&mysqld_bin)
            .env("HOME", install_dir)
            .arg("--initialize")
            .arg(format!("--datadir={}", data_dir.display()))
            .arg(format!("--basedir={}", install_dir.display()))
            .output()?
    };
    
    if !output.status.success() {
        eprintln!("⚠️  MariaDB initialization output: {}", String::from_utf8_lossy(&output.stderr));
        // Non-fatal error, as initialization might fail for various reasons
    } else {
        eprintln!("✅ MariaDB data directory initialized successfully");
    }
    
    // Create default configuration file
    create_default_config(install_dir, &data_dir)?;
    
    // Print connection information
    print_connection_info(install_dir)?;
    
    Ok(())
}

fn create_default_config(install_dir: &Path, data_dir: &Path) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let config_path = install_dir.join(if cfg!(windows) { "my.ini" } else { "my.cnf" });

    let config_content = format!(
        r#"[mysqld]
# MariaDB Server Configuration
port = 3306
datadir = {}
basedir = {}

# Security
default_authentication_plugin = mysql_native_password

# Network
max_connections = 100
max_connect_errors = 10

# Performance
innodb_buffer_pool_size = 128M
"#,
        data_dir.display(),
        install_dir.display()
    );
    
    let mut config_file = File::create(&config_path)?;
    config_file.write_all(config_content.as_bytes())?;
    
    eprintln!("📋 Created default configuration at: {}", config_path.display());
    
    Ok(())
}

fn print_connection_info(install_dir: &Path) -> Result<()> {
    eprintln!("\n🎉 MariaDB {} installation completed!", install_dir.file_name().unwrap_or(std::ffi::OsStr::new("<unknown>")).to_string_lossy());
    eprintln!("\n🔧 To start MariaDB server:");
    if cfg!(windows) {
        eprintln!("   cd \"{}\"", install_dir.display());
        eprintln!("   .\\bin\\mysqld --defaults-file=my.ini --console");
    } else {
        eprintln!("   cd \"{}\"", install_dir.display());
        eprintln!("   ./bin/mysqld --defaults-file=my.cnf");
    }
    
    eprintln!("\n🔐 Initial root password is usually stored in:");
    eprintln!("   Windows: Look in the error log at: {}\\data\\*.err", install_dir.display());
    eprintln!("   Linux/Mac: Look in the error log at: {}/data/*.err", install_dir.display());
    
    eprintln!("\n💡 To connect to MariaDB:");
    if cfg!(windows) {
        eprintln!("   .\\bin\\mysql -h 127.0.0.1 -P 3306 -u root -p");
    } else {
        eprintln!("   ./bin/mysql -h 127.0.0.1 -P 3306 -u root -p");
    }
    
    eprintln!("\n⚠️  Remember to change the root password after first login!");
    eprintln!("\n💡 Note: MariaDB is fully compatible with MySQL commands and syntax.");
    
    Ok(())
}