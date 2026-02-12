// src/localization.rs
use std::collections::HashMap;

/// 简单的本地化实现
pub struct Localizer {
    lang: String,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Localizer {
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        
        // 英文翻译
        let mut en_translations = HashMap::new();
        en_translations.insert("app_description".to_string(), "A unified development environment manager".to_string());
        en_translations.insert("command_init_description".to_string(), "Initialize enman environment".to_string());
        en_translations.insert("command_install_description".to_string(), "Install a specific version of a tool".to_string());
        en_translations.insert("command_global_description".to_string(), "Set global default tool version".to_string());
        en_translations.insert("command_list_description".to_string(), "List installed or available versions".to_string());
        en_translations.insert("command_use_description".to_string(), "Temporarily switch tool version for current session".to_string());
        en_translations.insert("command_uninstall_description".to_string(), "Uninstall a specific version of a tool".to_string());
        en_translations.insert("command_config_description".to_string(), "Manage project-level configuration".to_string());
        en_translations.insert("arg_tool_version_help".to_string(), "Tool and version in format: tool@version (e.g., node@20.10.0)".to_string());
        en_translations.insert("arg_tool_help".to_string(), "Tool name (e.g., node, python, java)".to_string());
        en_translations.insert("arg_version_help".to_string(), "Version (e.g., 18.17.0, latest)".to_string());
        en_translations.insert("arg_remote_help".to_string(), "Show remote available versions".to_string());
        en_translations.insert("arg_available_help".to_string(), "Show all available tools".to_string());
        en_translations.insert("Installing".to_string(), "Installing".to_string());
        en_translations.insert("Switching to".to_string(), "Switching to".to_string());
        en_translations.insert("Warning: Could not stop current MySQL service:".to_string(), "Warning: Could not stop current MySQL service:".to_string());
        en_translations.insert("Stopped previous MySQL service".to_string(), "Stopped previous MySQL service".to_string());
        en_translations.insert("Warning: Could not start new MySQL service:".to_string(), "Warning: Could not start new MySQL service:".to_string());
        en_translations.insert("Please start MySQL manually using: .\\bin\\mysqld --defaults-file=my.ini --console".to_string(), "Please start MySQL manually using: .\\bin\\mysqld --defaults-file=my.ini --console".to_string());
        en_translations.insert("Started new MySQL service for version".to_string(), "Started new MySQL service for version".to_string());
        en_translations.insert("Successfully set global".to_string(), "Successfully set global".to_string());
        en_translations.insert("to version".to_string(), "to version".to_string());
        en_translations.insert("Latest versions".to_string(), "Latest versions".to_string());
        en_translations.insert("Failed to fetch".to_string(), "Failed to fetch".to_string());
        en_translations.insert("versions".to_string(), "versions".to_string());
        en_translations.insert("Common Node.js versions".to_string(), "Common Node.js versions".to_string());
        en_translations.insert("Common Java versions".to_string(), "Common Java versions".to_string());
        en_translations.insert("Common Python versions".to_string(), "Common Python versions".to_string());
        en_translations.insert("Supported".to_string(), "Supported".to_string());
        en_translations.insert("Tool is not supported for remote version listing".to_string(), "Tool is not supported for remote version listing".to_string());
        en_translations.insert("Available tools that can be installed:".to_string(), "Available tools that can be installed:".to_string());
        en_translations.insert("To see available versions for a tool, use".to_string(), "To see available versions for a tool, use".to_string());
        en_translations.insert("No versions of".to_string(), "No versions of".to_string());
        en_translations.insert("installed".to_string(), "installed".to_string());
        en_translations.insert("local".to_string(), "local".to_string());
        en_translations.insert("global".to_string(), "global".to_string());
        en_translations.insert("not installed".to_string(), "not installed".to_string());
        en_translations.insert("No global versions set.".to_string(), "No global versions set.".to_string());
        
        // 中文翻译
        let mut zh_translations = HashMap::new();
        zh_translations.insert("app_description".to_string(), "统一开发环境管理器".to_string());
        zh_translations.insert("command_init_description".to_string(), "初始化 enman 环境".to_string());
        zh_translations.insert("command_install_description".to_string(), "安装指定版本的工具".to_string());
        zh_translations.insert("command_global_description".to_string(), "设置全局默认工具版本".to_string());
        zh_translations.insert("command_list_description".to_string(), "列出已安装或可用的版本".to_string());
        zh_translations.insert("command_use_description".to_string(), "临时切换当前会话的工具版本".to_string());
        zh_translations.insert("command_uninstall_description".to_string(), "卸载指定版本的工具".to_string());
        zh_translations.insert("command_config_description".to_string(), "管理项目级配置".to_string());
        zh_translations.insert("arg_tool_version_help".to_string(), "工具和版本，格式：tool@version (例如，node@20.10.0)".to_string());
        zh_translations.insert("arg_tool_help".to_string(), "工具名称 (例如，node, python, java)".to_string());
        zh_translations.insert("arg_version_help".to_string(), "版本 (例如，18.17.0, latest)".to_string());
        zh_translations.insert("arg_remote_help".to_string(), "显示远程可用版本".to_string());
        zh_translations.insert("arg_available_help".to_string(), "显示所有可用工具".to_string());
        zh_translations.insert("Installing".to_string(), "正在安装".to_string());
        zh_translations.insert("Switching to".to_string(), "正在切换到".to_string());
        zh_translations.insert("Warning: Could not stop current MySQL service:".to_string(), "警告：无法停止当前 MySQL 服务：".to_string());
        zh_translations.insert("Stopped previous MySQL service".to_string(), "已停止之前的 MySQL 服务".to_string());
        zh_translations.insert("Warning: Could not start new MySQL service:".to_string(), "警告：无法启动新的 MySQL 服务：".to_string());
        zh_translations.insert("Please start MySQL manually using: .\\bin\\mysqld --defaults-file=my.ini --console".to_string(), "请手动启动 MySQL：.\\bin\\mysqld --defaults-file=my.ini --console".to_string());
        zh_translations.insert("Started new MySQL service for version".to_string(), "已为版本启动新的 MySQL 服务".to_string());
        zh_translations.insert("Successfully set global".to_string(), "成功设置全局".to_string());
        zh_translations.insert("to version".to_string(), "到版本".to_string());
        zh_translations.insert("Latest versions".to_string(), "最新版本".to_string());
        zh_translations.insert("Failed to fetch".to_string(), "获取失败".to_string());
        zh_translations.insert("versions".to_string(), "版本".to_string());
        zh_translations.insert("Common Node.js versions".to_string(), "常见 Node.js 版本".to_string());
        zh_translations.insert("Common Java versions".to_string(), "常见 Java 版本".to_string());
        zh_translations.insert("Common Python versions".to_string(), "常见 Python 版本".to_string());
        zh_translations.insert("Supported".to_string(), "支持的".to_string());
        zh_translations.insert("Tool is not supported for remote version listing".to_string(), "该工具不支持远程版本查询".to_string());
        zh_translations.insert("Available tools that can be installed:".to_string(), "可安装的工具：".to_string());
        zh_translations.insert("To see available versions for a tool, use".to_string(), "查看工具的可用版本，请使用".to_string());
        zh_translations.insert("No versions of".to_string(), "没有安装".to_string());
        zh_translations.insert("installed".to_string(), "的版本".to_string());
        zh_translations.insert("local".to_string(), "本地".to_string());
        zh_translations.insert("global".to_string(), "全局".to_string());
        zh_translations.insert("not installed".to_string(), "未安装".to_string());
        zh_translations.insert("No global versions set.".to_string(), "未设置全局版本。".to_string());
        
        translations.insert("en".to_string(), en_translations);
        translations.insert("zh".to_string(), zh_translations);
        
        // 检测系统语言
        let lang = detect_language();
        
        Self {
            lang,
            translations,
        }
    }
    
    pub fn t(&self, key: &str) -> String {
        if let Some(lang_map) = self.translations.get(&self.lang) {
            if let Some(text) = lang_map.get(key) {
                return text.clone();
            }
        }
        
        // 回退到英文
        if let Some(en_map) = self.translations.get("en") {
            if let Some(text) = en_map.get(key) {
                return text.clone();
            }
        }
        
        // 如果找不到翻译，则返回键值本身
        key.to_string()
    }
}

fn detect_language() -> String {
    // 尝试从环境变量检测语言
    if let Ok(lang) = std::env::var("LANG") {
        if lang.starts_with("zh") {
            return "zh".to_string();
        }
    }
    
    // Windows 系统检测
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use winapi::um::winnls::GetUserDefaultLocaleName;
        
        let mut buffer: [u16; 85] = [0; 85]; // LOCALE_NAME_MAX_LENGTH
        let len = unsafe {
            GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32)
        };
        
        if len > 0 {
            // 转换为字符串，去掉尾随的null字节
            let locale_name = OsString::from_wide(&buffer[..(len as usize - 1)])
                .to_string_lossy()
                .to_lowercase();
                
            if locale_name.starts_with("zh") {
                return "zh".to_string();
            }
        }
    }
    
    // 默认为英文
    "en".to_string()
}

impl Default for Localizer {
    fn default() -> Self {
        Self::new()
    }
}

// 全局本地化实例
static mut LOCALIZER: Option<Localizer> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub fn get_localizer() -> &'static Localizer {
    unsafe {
        INIT.call_once(|| {
            LOCALIZER = Some(Localizer::new());
        });
        LOCALIZER.as_ref().unwrap()
    }
}

#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        $crate::localization::get_localizer().t($key)
    };
}