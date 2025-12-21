use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Clones or updates a Git repository and checks out a specific reference,
/// and sets up origin and upstream remotes.
pub fn manage_git_repo(
    repo_url: &str,
    target_dir: &Path,
    reference: &str,
    github_org: Option<&str>,
    repo_fork_mapping: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let repo_name = target_dir.file_name().context("Target directory has no file name")?.to_str().context("Repository name is not valid UTF-8")?;

    // Determine the fork URL
    let fork_url = if let Some(explicit_fork_url) = repo_fork_mapping.get(repo_url) {
        explicit_fork_url.clone()
    } else if let Some(org) = github_org {
        // Assume GitHub for now for simple fork URL generation
        let upstream_host = url::Url::parse(repo_url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .context("Could not parse upstream host from repo URL")?;
        format!("https://{}/{}/{}", upstream_host, org, repo_name)
    } else {
        repo_url.to_string() // No org, use original as fork (effectively no fork)
    };

    if !target_dir.exists() {
        println!("Cloning fork {} to {}...", fork_url, target_dir.display());
        Command::new("git")
            .arg("clone")
            .arg(&fork_url)
            .arg(target_dir)
            .status()
            .context(format!("Failed to clone repository {}", fork_url))?;
    } else {
        println!("Repository already exists at {}. Fetching latest...", target_dir.display());
        Command::new("git")
            .arg("-C") // Run command in target_dir
            .arg(target_dir)
            .arg("fetch")
            .arg("origin")
            .status()
            .context(format!("Failed to fetch origin for {}", target_dir.display()))?;
        
        // Ensure origin is correctly set to the fork URL
        Command::new("git")
            .arg("-C")
            .arg(target_dir)
            .arg("remote")
            .arg("set-url")
            .arg("origin")
            .arg(&fork_url)
            .status()
            .context(format!("Failed to set origin URL to {}", fork_url))?;
    }

    // Set upstream remote
    println!("Setting upstream remote to {}...", repo_url);
    Command::new("git")
        .arg("-C")
        .arg(target_dir)
        .arg("remote")
        .arg("add")
        .arg("upstream")
        .arg(repo_url)
        .status()
        .or_else(|e| {
            // If upstream already exists, try to set its URL
            if e.to_string().contains("already exists") {
                Command::new("git")
                    .arg("-C")
                    .arg(target_dir)
                    .arg("remote")
                    .arg("set-url")
                    .arg("upstream")
                    .arg(repo_url)
                    .status()
                    .context(format!("Failed to set upstream URL to {}", repo_url))
            } else {
                Err(e).context("Failed to add upstream remote")
            }
        })?;


    println!("Checking out {} in {}...", reference, target_dir.display());
    Command::new("git")
        .arg("-C") // Run command in target_dir
        .arg(target_dir)
        .arg("checkout")
        .arg(reference)
        .status()
        .context(format!("Failed to checkout {} in {}", reference, target_dir.display()))?;

    Ok(())
}
