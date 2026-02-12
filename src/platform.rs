// src/platform.rs
#[derive(Debug, Clone)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

#[derive(Debug, Clone)]
pub enum Os {
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, Clone)]
pub enum Arch {
    X64,
    Arm64,
}

impl Platform {
    pub fn detect() -> Self {
        let os = if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        };

        let arch = if cfg!(target_arch = "x86_64") {
            Arch::X64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Arm64
        } else {
            panic!("Unsupported architecture");
        };

        Self { os, arch }
    }
}