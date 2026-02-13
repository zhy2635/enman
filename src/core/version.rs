use semver::{Version, VersionReq};

/// 比较两个版本字符串
/// 返回值：Ordering::Greater 如果 a > b
///         Ordering::Less 如果 a < b
///         Ordering::Equal 如果 a == b
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    // 尝试将版本字符串解析为SemVer格式进行比较
    let version_a = Version::parse(a.strip_prefix('v').unwrap_or(a));
    let version_b = Version::parse(b.strip_prefix('v').unwrap_or(b));
    
    match (version_a, version_b) {
        (Ok(ver_a), Ok(ver_b)) => ver_a.cmp(&ver_b),
        _ => {
            // 如果无法解析为semver格式，则按字典序比较
            a.cmp(b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        use std::cmp::Ordering;
        
        assert_eq!(compare_versions("1.0.0", "1.0.1"), Ordering::Less);
        assert_eq!(compare_versions("1.0.1", "1.0.0"), Ordering::Greater);
        assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
        assert_eq!(compare_versions("2.0.0", "1.9.9"), Ordering::Greater);
    }
}