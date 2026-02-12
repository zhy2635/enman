// src/download.rs
use crate::{tool::Tool, platform::Platform};

#[derive(Debug)]
pub struct DownloadInfo {
    pub primary_url: String,
    pub fallback_urls: Vec<String>,
    pub file_type: FileType,
}

#[derive(Debug)]
pub enum FileType {
    Zip,
    TarGz,
    TarXz,
    Exe, // Windows installer
}

pub trait Downloadable {
    fn get_download_info(&self, version: &str, platform: &Platform) -> Option<DownloadInfo>;
}

impl Downloadable for Tool {
    fn get_download_info(&self, version: &str, platform: &Platform) -> Option<DownloadInfo> {
        match self {
            Tool::Node => node_download_info(version, platform),
            Tool::Java => java_download_info(version, platform),
            Tool::Python => python_download_info(version, platform),
            Tool::MySql => mysql_download_info(version, platform),
        }
    }
}