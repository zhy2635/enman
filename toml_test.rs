use std::collections::HashMap;
use toml::Value;

fn main() {
    let content = "[tools]\npython = \"3.10.9\"";
    println!("Content: {}", content);
    
    let toml_config: HashMap<String, Value> = toml::from_str(content).unwrap();
    if let Some(tools_obj) = toml_config.get("tools") {
        if let Some(tools) = tools_obj.as_table() {
            if let Some(version_value) = tools.get("python") {
                if let Some(version_str) = version_value.as_str() {
                    println!("Parsed version: '{}', length: {}", version_str, version_str.len());
                    println!("Characters: {:?}", version_str.chars().map(|c| format!("'{}'({})", c, c as u32)).collect::<Vec<_>>());
                    
                    // 检查是否包含引号
                    if version_str.starts_with('"') && version_str.ends_with('"') && version_str.len() >= 2 {
                        let stripped = &version_str[1..version_str.len()-1];
                        println!("Stripped version: '{}', length: {}", stripped, stripped.len());
                    }
                }
            }
        }
    }
}