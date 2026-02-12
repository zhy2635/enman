// src/downloader/mysql.rs
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::fs;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use std::process::Command;
use std::io::{self, Write};
use std::process;

// 只保留必要依赖
use rpassword;

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
        "tar.xz"
    };

    Ok((os, arch, ext))
}

fn get_download_urls(version: &str) -> Result<Vec<String>> {
    let (os, arch, ext) = detect_platform()?;
    
    let filename = if os == "win" {
        format!("mysql-{}-{}.{}", version, "winx64", ext)
    } else {
        format!("mysql-{}-{}{}.{}", version, arch, if arch == "x64" { "" } else { "" }, ext)
    };
    
    let mut urls = Vec::new();
    let major_version = version.rsplit_once('.').unwrap_or((&version, "")).0;

    if os == "win" {
        urls.push(format!("https://mirrors.aliyun.com/mysql/MySQL-{}/{}", major_version, filename));
        urls.push(format!("https://dev.mysql.com/get/Downloads/MySQL-{}/{}", major_version, filename));
        urls.push(format!("https://cdn.mysql.com/Downloads/MySQL-{}/{}", major_version, filename));
        urls.push(format!("https://archives.mysql.com/Downloads/MySQL-{}/{}", major_version, filename));
    } else {
        urls.push(format!("https://dev.mysql.com/get/Downloads/MySQL-{}/{}", major_version, filename));
        urls.push(format!("https://cdn.mysql.com/Downloads/MySQL-{}/{}", major_version, filename));
    }

    // MariaDB fallback
    if os == "win" {
        urls.push(format!(
            "https://mirrors.tuna.tsinghua.edu.cn/mariadb/mariadb-{}/mariadb-{}-{}.{}",
            version, version, "winx64", "zip"
        ));
    } else {
        let platform_str = if os == "macos" { "x86_64".to_string() } else { format!("{}-{}", os, arch) };
        urls.push(format!(
            "https://mirrors.tuna.tsinghua.edu.cn/mariadb/mariadb-{}/bintar-{}/mariadb-{}-{}-{}.{}",
            version, platform_str, version, os, arch, "tar.gz"
        ));
    }

    Ok(urls)
}

async fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; enman/1.0)")
        .build()
        .context("Failed to build HTTP client")?;

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

fn extract_and_flatten(archive: &Path, dest: &Path, format: &str) -> Result<()> {
    fs::create_dir_all(dest)?;
    let temp_extract = TempDir::new()?;

    eprintln!("🔧 Extracting MySQL as {}...", format);

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

// ===== 关键：生成 my.ini 并禁用 EventLog =====
fn create_default_config(install_dir: &Path, data_dir: &Path, port: u16) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let config_path = install_dir.join("my.ini"); // Windows uses my.ini

    let config_content = format!(
        r#"[mysqld]
port = {}
datadir = {}
basedir = {}
default_authentication_plugin = mysql_native_password
max_connections = 100
max_connect_errors = 10
innodb_buffer_pool_size = 128M
# Disable Windows EventLog to avoid permission issues
log-error = {}
general_log = 0
event-scheduler = off

[client]
port = {}
"#,
        port,
        data_dir.display(),
        install_dir.display(),
        data_dir.join("mysql_error.log").display(),
        port
    );
    
    let mut config_file = File::create(&config_path)?;
    config_file.write_all(config_content.as_bytes())?;
    
    eprintln!("📋 Updated configuration at: {}", config_path.display());
    Ok(())
}

// ===== 初始化数据目录 =====
async fn initialize_mysql(install_dir: &Path, _version: &str) -> Result<()> {
    eprintln!("🔐 Initializing MySQL data directory (insecure mode)...");
    
    let data_dir = install_dir.join("data");
    fs::create_dir_all(&data_dir)?;
    
    let mysqld_bin = install_dir.join("bin").join("mysqld.exe");
    
    let output = Command::new(&mysqld_bin)
        .arg("--initialize-insecure")
        .arg(format!("--datadir={}", data_dir.display()))
        .arg(format!("--basedir={}", install_dir.display()))
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "MySQL initialization failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    eprintln!("✅ MySQL initialized with empty root password");
    create_default_config(install_dir, &data_dir, 3306)?;
    Ok(())
}

// ===== 核心：通过 --init-file 配置用户 =====
async fn configure_mysql_interactive(install_dir: &Path) -> Result<u16> {
    println!("\n🔧 Configure your MySQL instance:");

    let port = loop {
        print!("Enter port (default 3306): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            break 3306;
        }
        match input.parse::<u16>() {
            Ok(0) => eprintln!("⚠️  Port must be between 1 and 65535"),
            Ok(p) => break p,
            Err(_) => eprintln!("⚠️  Please enter a valid number"),
        }
    };

    let root_password = loop {
        print!("Enter root password (min 8 chars): ");
        io::stdout().flush()?;
        let pwd = rpassword::read_password()?;
        if pwd.len() >= 8 {
            break pwd;
        }
        eprintln!("⚠️  Password must be at least 8 characters");
    };

    print!("Create additional user? (y/N): ");
    io::stdout().flush()?;
    let mut create_user = String::new();
    io::stdin().read_line(&mut create_user)?;
    let extra_user = if create_user.trim().to_lowercase() == "y" {
        print!("Username: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        let username = username.trim().to_string();

        if username == "root" {
            eprintln!("⚠️  Cannot create user named 'root'. Skipping.");
            None
        } else {
            print!("Password: ");
            io::stdout().flush()?;
            let password = rpassword::read_password()?;
            Some((username, password))
        }
    } else {
        None
    };

    let data_dir = install_dir.join("data");
    create_default_config(install_dir, &data_dir, port)?;

    // 生成 init.sql
    let init_sql_path = install_dir.join("init.sql");
    let mut sql = String::new();

    // MySQL 5.7 兼容：使用 SET PASSWORD = PASSWORD('...')
    sql.push_str(&format!(
        "SET PASSWORD FOR 'root'@'localhost' = PASSWORD('{}');\n",
        root_password.replace("'", "''")
    ));

    if let Some((username, password)) = extra_user {
        sql.push_str(&format!(
            "CREATE USER '{}'@'%' IDENTIFIED BY '{}';\n",
            username.replace("'", "''"),
            password.replace("'", "''")
        ));
        sql.push_str(&format!(
            "GRANT ALL PRIVILEGES ON *.* TO '{}'@'%';\n",
            username.replace("'", "''")
        ));
    }

    sql.push_str("FLUSH PRIVILEGES;\n");

    std::fs::write(&init_sql_path, sql)?;
    eprintln!("📝 Created init script at: {}", init_sql_path.display());

    // 启动一次 mysqld 执行 init.sql
    let mysqld_bin = install_dir.join("bin").join("mysqld.exe");
    let config_path = install_dir.join("my.ini");

    eprintln!("🔄 Running MySQL with --init-file to apply settings...");
    let status = Command::new(mysqld_bin)
        .arg(format!("--defaults-file={}", config_path.display()))
        .arg(format!("--init-file={}", init_sql_path.display()))
        .arg("--console")
        .status()?;

    // 清理临时文件
    let _ = std::fs::remove_file(init_sql_path);

    if !status.success() {
        return Err(anyhow::anyhow!(
            "MySQL --init-file failed. Check {}/mysql_error.log for details.",
            data_dir.display()
        ));
    }

    eprintln!("✅ Configuration applied successfully!");
    Ok(port)
}

// ===== MySQL 服务管理功能 =====
pub fn stop_current_mysql_service() -> Result<()> {
    eprintln!("🛑 Stopping any running MySQL service...");
    
    // 尝试找到并终止任何正在运行的 mysqld 进程
    let result = if cfg!(windows) {
        // 使用 wmic 命令获取详细进程信息，然后终止
        let output = Command::new("wmic")
            .args(&["process", "where", "name='mysqld.exe'", "call", "terminate"])
            .output();
            
        match output {
            Ok(output_result) => {
                if output_result.status.success() {
                    eprintln!("✅ All MySQL processes terminated");
                    Ok(())
                } else {
                    // 如果 wmic 失败，回退到 taskkill
                    let taskkill_result = Command::new("taskkill")
                        .args(&["/f", "/im", "mysqld.exe"])
                        .output();
                    
                    match taskkill_result {
                        Ok(tk_output) => {
                            if tk_output.status.success() || 
                               String::from_utf8_lossy(&tk_output.stderr).contains("not found") ||
                               String::from_utf8_lossy(&tk_output.stdout).contains("not found") {
                                eprintln!("✅ MySQL processes stopped or were not running");
                                Ok(())
                            } else {
                                eprintln!("⚠️ Error stopping MySQL process: {}", String::from_utf8_lossy(&tk_output.stderr));
                                Ok(()) // 不返回错误，因为这不是致命错误
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️ Could not stop MySQL process: {}", e);
                            Ok(())
                        }
                    }
                }
            }
            Err(wmic_e) => {
                eprintln!("⚠️ WMIC command failed: {}, trying taskkill instead", wmic_e);
                // 回退到原来的 taskkill 方法
                let taskkill_result = Command::new("taskkill")
                    .args(&["/f", "/im", "mysqld.exe"])
                    .output();
                
                match taskkill_result {
                    Ok(output) => {
                        if output.status.success() || 
                           String::from_utf8_lossy(&output.stderr).contains("not found") ||
                           String::from_utf8_lossy(&output.stdout).contains("not found") {
                            eprintln!("✅ MySQL processes stopped or were not running");
                            Ok(())
                        } else {
                            eprintln!("⚠️ Error stopping MySQL process: {}", String::from_utf8_lossy(&output.stderr));
                            Ok(()) // 不返回错误，因为这不是致命错误
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ Could not stop MySQL process: {}", e);
                        Ok(())
                    }
                }
            }
        }
    } else {
        let output = Command::new("pkill")
            .args(&["-f", "mysqld"])
            .output();
        
        match output {
            Ok(output_result) => {
                if output_result.status.success() || 
                   String::from_utf8_lossy(&output_result.stderr).contains("not found") ||
                   String::from_utf8_lossy(&output_result.stdout).contains("not found") {
                    eprintln!("✅ MySQL processes stopped or were not running");
                } else {
                    let stderr = String::from_utf8_lossy(&output_result.stderr);
                    if stderr.contains("not found") || stderr.contains("no process") {
                        eprintln!("✅ No MySQL processes were running");
                    } else {
                        eprintln!("⚠️ Error stopping MySQL process: {}", stderr);
                    }
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("⚠️ Could not stop MySQL process: {}", e);
                Ok(())
            }
        }
    };

    result  // 返回计算结果而不是嵌套的 Result
}

pub fn start_mysql_service(install_dir: &Path) -> Result<()> {
    eprintln!("🚀 Starting MySQL service for version located at: {}", install_dir.display());
    
    let mysqld_bin = install_dir.join("bin").join("mysqld.exe");
    let config_path = install_dir.join("my.ini");
    
    if !mysqld_bin.exists() {
        return Err(anyhow::anyhow!("mysqld binary not found at {:?}", mysqld_bin));
    }
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Configuration file not found at {:?}", config_path));
    }

    // 在后台启动MySQL服务
    let child = Command::new(&mysqld_bin)
        .arg(format!("--defaults-file={}", config_path.display()))
        .spawn()?;

    // 检查进程是否成功启动（不等待完成）
    if child.id() > 0 {
        eprintln!("✅ MySQL service started successfully (PID: {})", child.id());
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to start MySQL service"))
    }
}

// ===== 主安装函数 =====
pub async fn install(version: &str, install_dir: &Path) -> Result<()> {
    let mysql_bin = install_dir.join("bin").join("mysqld.exe");

    if mysql_bin.exists() {
        println!("⚠️  MySQL {} already installed", version);
        return Ok(());
    }

    let (_os, _arch, ext) = detect_platform()?;
    let urls = get_download_urls(version)?;
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join(format!("mysql.{}", ext));

    let mut success = false;
    for (i, url) in urls.iter().enumerate() {
        if i >= 7 { break; }
        let source = if i == 0 { "Alibaba Cloud Mirror" } else if i <= 3 { "Mirror" } else { "MariaDB Fallback" };
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
        eprintln!("❌ Unable to automatically download MySQL.");
        eprintln!("\n💡 Manual installation steps:");
        eprintln!("   1. Visit: https://dev.mysql.com/downloads/mysql/");
        eprintln!("   2. Download the appropriate version for your OS");
        eprintln!("   3. Extract to: {}", install_dir.display());
        eprintln!("   4. Run: enman global mysql@{}", version);
        return Err(anyhow::anyhow!("Automatic download failed."));
    }

    extract_and_flatten(&archive_path, install_dir, ext)
        .context("Failed to extract MySQL")?;

    if !mysql_bin.exists() {
        return Err(anyhow::anyhow!("Verification failed: mysqld binary not found at {:?}", mysql_bin));
    }

    eprintln!("✨ MySQL {} installed to {}", version, install_dir.display());
    
    initialize_mysql(install_dir, version).await?;
    let final_port = configure_mysql_interactive(install_dir).await?;

    eprintln!("\n🎉 MySQL {} is ready on port {}!", version, final_port);
    eprintln!("\n💡 To start MySQL server:");
    eprintln!("   cd \"{}\"", install_dir.display());
    eprintln!("   .\\bin\\mysqld --defaults-file=my.ini --console");
    eprintln!("\n🔐 Connect with:");
    eprintln!("   .\\bin\\mysql -h 127.0.0.1 -P {} -u root -p", final_port);
    
    Ok(())
}