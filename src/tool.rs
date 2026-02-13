// src/tool.rs
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    Node,
    Java,
    Python,
    MySql,
    Redis,  // 添加Redis支持
}

impl Tool {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "node" => Some(Tool::Node),
            "java" => Some(Tool::Java),
            "python" => Some(Tool::Python),
            "mysql" => Some(Tool::MySql),
            "redis" => Some(Tool::Redis),  // 添加Redis支持
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tool::Node => "node",
            Tool::Java => "java",
            Tool::Python => "python",
            Tool::MySql => "mysql",
            Tool::Redis => "redis",  // 添加Redis支持
        }
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}