use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn format_rust_file(path: &Path) -> Result<()> {
    let output = Command::new("rustfmt")
        .arg(path)
        .output()
        .context(format!("Failed to execute rustfmt on {}", path.display()))?;

    if !output.status.success() {
        eprintln!("WARNING: rustfmt failed on {}:", path.display());
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}
