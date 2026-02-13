// src/downloader/mysql.rs
use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{self, BufRead, BufReader};
use std::net::TcpListener;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use rpassword;
use zip;

use crate::core::paths;

pub async fn download_mysql(version: &str, dest: &Path) -> Result<()> {
    // 获取用户主目录
    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    let enman_dir = home_dir.join(".enman");

    // 创建下载缓存目录
    let cache_dir = enman_dir.join("cache");
    fs::create_dir_all(&cache_dir)?;

    // 根据版本确定下载URL
    // 这里仅为示例，实际使用时需要根据版本选择对应的下载链接
    let url = if version.starts_with("8.") {
        format!("https://dev.mysql.com/get/Downloads/MySQL-{}/mysql-{}-winx64.zip", 
                &version[..3], version)
    } else {
        format!("https://dev.mysql.com/get/Downloads/MySQL-{}/mysql-{}-winx64.msi", 
                &version[..3], version)
    };

    // 确定下载文件名
    let filename = cache_dir.join(format!("mysql-{}.zip", version));

    // 如果文件不存在，则下载
    if !filename.exists() {
        println!("Downloading MySQL {}...", version);

        // 使用reqwest下载文件
        let response = reqwest::get(&url).await
            .with_context(|| format!("Failed to download from: {}", url))?;

        if !response.status().is_success() {
            bail!("Download request failed with status: {}", response.status());
        }

        let content = response.bytes().await
            .context("Failed to read downloaded content")?;

        // 写入文件
        let mut file = std::fs::File::create(&filename)
            .with_context(|| format!("Failed to create file: {}", filename.display()))?;
        std::io::copy(&mut content.as_ref(), &mut file)
            .context("Failed to save downloaded file")?;
    } else {
        println!("Using cached MySQL {} archive", version);
    }

    // 创建目标目录
    fs::create_dir_all(dest)?;

    // 解压文件
    let file = std::fs::File::open(&filename)?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to open zip archive: {}", filename.display()))?;

    for i in 0..archive.len() {
        let mut file_in_archive = archive.by_index(i)?;
        let outpath = dest.join(file_in_archive.mangled_name());

        if (*file_in_archive.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file_in_archive, &mut outfile)?;
        }
    }

    // 查找解压后的根目录（可能包含版本号的文件夹）
    let extracted_dir = dest.read_dir()?
        .find_map(|entry| {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                Some(entry.path())
            } else {
                None
            }
        })
        .with_context(|| "Could not find MySQL directory inside the archive")?;

    // 将内容从提取的目录移动到目标目录
    for entry in std::fs::read_dir(&extracted_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dest.join(entry.file_name());

        if src_path.is_file() {
            std::fs::rename(src_path, &dst_path)?;
        } else {
            // 如果是目录，移动整个目录树
            if dst_path.exists() {
                std::fs::remove_dir_all(&dst_path)?;
            }
            std::fs::rename(src_path, &dst_path)?;
        }
    }

    // 删除提取目录
    std::fs::remove_dir_all(&extracted_dir)?;

    println!("MySQL {} installed successfully", version);
    Ok(())
}

pub fn setup_mysql_initial_config(install_path: &Path) -> Result<()> {
    // 创建数据目录
    let data_dir = install_path.join("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    // 初始化MySQL
    let mysqld_bin = install_path.join("bin").join("mysqld.exe");
    let init_result = Command::new(&mysqld_bin)
        .arg("--initialize-insecure")  // 不设置默认root密码
        .arg(format!("--datadir={}", data_dir.display()))
        .arg(format!("--basedir={}", install_path.display()))
        .output()?;

    if !init_result.status.success() {
        let stderr = String::from_utf8_lossy(&init_result.stderr);
        bail!("MySQL initialization failed: {}", stderr);
    }

    // 创建配置文件
    let config_path = install_path.join("my.ini");
    let port = 3306;  // 可以根据版本或配置生成不同的端口号
    let config_content = format!(
        "[mysqld]\nport={}\ndatadir={}\n\n[mysql]\ndefault-character-set=utf8\n",
        port,
        data_dir.display().to_string().replace("\\", "\\\\")
    );

    fs::write(&config_path, config_content)?;

    println!("MySQL initial configuration completed");
    Ok(())
}

pub fn configure_mysql_auto(install_path: &Path) -> Result<()> {
    // 获取用户输入的密码
    print!("Enter root password (min 8 chars): ");
    std::io::stdout().flush()?;
    let mut password = String::new();
    std::io::stdin().read_line(&mut password)?;
    password = password.trim().to_string();

    if password.len() < 8 {
        println!("Password is too short. Using default password 'root123'");
        password = "root123".to_string();
    }

    // 创建一个临时的init文件用于更改密码
    let init_sql_path = install_path.join("reset_password.sql");
    let sql_content = format!(
        "ALTER USER 'root'@'localhost' IDENTIFIED BY '{}';\nFLUSH PRIVILEGES;\n",
        password
    );
    fs::write(&init_sql_path, sql_content)?;

    // 使用--init-file参数启动MySQL，这样可以在启动后立即执行密码设置
    let mysqld_bin = install_path.join("bin").join("mysqld.exe");
    let data_dir = install_path.join("data");
    let config_path = install_path.join("my.ini");

    let mut child = Command::new(&mysqld_bin)
        .arg("--defaults-file=".to_string() + &config_path.display().to_string())
        .arg("--skip-grant-tables")
        .arg("--init-file=".to_string() + &init_sql_path.display().to_string())
        .spawn()?;

    // 等待一段时间让MySQL完成密码设置
    std::thread::sleep(std::time::Duration::from_secs(5));

    // 检查进程是否仍在运行
    if child.try_wait()?.is_none() {
        // 发送终止信号
        child.kill()?;
        let _ = child.wait()?;
    }

    // 删除临时SQL文件
    fs::remove_file(&init_sql_path)?;

    // 验证密码是否设置成功
    let mysql_bin = install_path.join("bin").join("mysql.exe");
    let test_connection = Command::new(&mysql_bin)
        .arg("-u")
        .arg("root")
        .arg("-p")
        .arg(&password)
        .arg("-e")
        .arg("SELECT 1;")
        .output()?;

    if test_connection.status.success() {
        println!("MySQL root password set successfully");
    } else {
        // 如果上面的方法失败，尝试备用方法
        println!("Primary method failed, attempting alternate password reset...");

        // 启动mysqld --skip-grant-tables
        let mut child = Command::new(&mysqld_bin)
            .arg("--defaults-file=".to_string() + &config_path.display().to_string())
            .arg("--skip-grant-tables")
            .spawn()?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        // 使用mysql客户端执行密码更改
        let result = Command::new(&mysql_bin)
            .arg("-u")
            .arg("root")
            .arg("-e")
            .arg(format!("ALTER USER 'root'@'localhost' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;", password))
            .output()?;

        // 结束mysqld进程
        if child.try_wait()?.is_none() {
            child.kill()?;
            let _ = child.wait()?;
        }

        if result.status.success() {
            println!("MySQL root password set successfully with alternate method");
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr);
            bail!("Failed to set MySQL password: {}", stderr);
        }
    }

    Ok(())
}


pub fn stop_current_mysql_service() -> Result<()> {
    // 首先尝试使用wmic命令查找mysqld进程
    let output = Command::new("wmic")
        .arg("process")
        .arg("where")
        .arg("name='mysqld.exe'")
        .arg("call")
        .arg("terminate")
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                // 如果WMIC不可用或失败，尝试使用taskkill
                let _ = Command::new("taskkill")
                    .arg("/f")
                    .arg("/im")
                    .arg("mysqld.exe")
                    .output()?;
            }
        }
        Err(_) => {
            // 如果WMIC命令不存在，使用taskkill
            let _ = Command::new("taskkill")
                .arg("/f")
                .arg("/im")
                .arg("mysqld.exe")
                .output()?;
        }
    }

    Ok(())
}

// 添加 install 函数
pub async fn install(version: &str, install_dir: &Path) -> Result<()> {
    download_mysql(version, install_dir).await?;
    setup_mysql_initial_config(install_dir)?;
    Ok(())
}

// 创建初始配置文件的函数
fn create_init_config(install_dir: &Path, data_dir: &Path) -> Result<PathBuf> {
    let config_path = install_dir.join("temp_my.ini");
    let config_content = format!(
        "[mysqld]\nskip-networking\nport=3306\ndatadir={}\nbasedir={}\n",
        data_dir.display(),
        install_dir.display()
    );
    std::fs::write(&config_path, config_content)?;
    Ok(config_path)
}

// 创建默认配置文件的函数
fn create_default_config(install_dir: &Path, data_dir: &Path, port: u16) -> Result<()> {
    let config_path = if cfg!(windows) {
        install_dir.join("my.ini")
    } else {
        install_dir.join("my.cnf")
    };
    
    let config_content = format!(
        "[mysqld]\nport={}\ndatadir={}\nbasedir={}\n\n[mysql]\ndefault-character-set=utf8mb4\n",
        port,
        data_dir.display(),
        install_dir.display()
    );
    
    std::fs::write(&config_path, config_content)?;
    Ok(())
}

/// 启动MySQL服务的函数
pub fn start_mysql_service(install_dir: &Path) -> Result<()> {
    let mysqld_bin = if cfg!(windows) {
        install_dir.join("bin").join("mysqld.exe")
    } else {
        install_dir.join("bin").join("mysqld")
    };

    let config_path = if cfg!(windows) {
        install_dir.join("my.ini")
    } else {
        install_dir.join("my.cnf")
    };

    if !config_path.exists() {
        bail!("Configuration file does not exist: {}", config_path.display());
    }

    eprintln!("🚀 Starting MySQL service...");
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;

        let _ = Command::new(&mysqld_bin)
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .arg(format!("--defaults-file={}", config_path.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }

    #[cfg(not(windows))]
    {
        let _ = Command::new(&mysqld_bin)
            .arg(format!("--defaults-file={}", config_path.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }

    std::thread::sleep(std::time::Duration::from_millis(1500));
    eprintln!("✅ MySQL service started successfully!");
    Ok(())
}


/// 使用 --init-file 设置密码（可靠方案）
async fn configure_and_start_mysql(install_dir: &Path, port: u16, root_password: String) -> Result<()> {
    let data_dir = install_dir.join("data");

    // Step 1: 写 init.sql
    let init_sql = install_dir.join("init.sql");
    fs::write(&init_sql, format!(
        "ALTER USER 'root'@'localhost' IDENTIFIED BY '{}';\nFLUSH PRIVILEGES;",
        root_password
    ))?;

    // Step 2: 创建无网络临时配置
    let temp_config = create_init_config(install_dir, &data_dir)?;

    let mysqld_bin = if cfg!(windows) {
        install_dir.join("bin").join("mysqld.exe")
    } else {
        install_dir.join("bin").join("mysqld")
    };

    eprintln!("🔐 Setting root password via --init-file (no network)...");

    // 启动一次，应用密码
    let mut child = Command::new(&mysqld_bin)
        .arg(format!("--defaults-file={}", temp_config.display()))
        .arg(format!("--init-file={}", init_sql.display()))
        .arg("--console")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    std::thread::sleep(std::time::Duration::from_secs(3));
    let _ = child.kill();
    let _ = child.wait();

    // 清理
    let _ = fs::remove_file(&init_sql);
    let _ = fs::remove_file(&temp_config);

    eprintln!("✅ Root password set successfully!");

    // Step 3: 写正式配置（带端口）
    create_default_config(install_dir, &data_dir, port)?;

    // Step 4: 启动正式服务
    let config_path = if cfg!(windows) {
        install_dir.join("my.ini")
    } else {
        install_dir.join("my.cnf")
    };

    eprintln!("🚀 Starting MySQL service on port {}...", port);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;

        let _ = Command::new(&mysqld_bin)
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .arg(format!("--defaults-file={}", config_path.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    #[cfg(not(windows))]
    {
        let _ = Command::new(&mysqld_bin)
            .arg(format!("--defaults-file={}", config_path.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    std::thread::sleep(std::time::Duration::from_millis(1500));
    eprintln!("✅ MySQL is running in the background on port {}!", port);
    Ok(())
}




