// src/cli/init.rs
use clap::Args;

#[derive(Args)]
pub struct InitArgs {}

pub async fn run(_args: InitArgs) -> anyhow::Result<()> {
    let paths = crate::core::paths::EnvManPaths::new()?;
    paths.ensure_dirs()?;
    println!("✅ enman initialized!");
    println!();
    println!("Add to your shell profile (~/.bashrc, ~/.zshrc, or $PROFILE):");
    println!("  export PATH=\"$HOME/.enman/shims:$PATH\"");
    Ok(())
}