use random_string::charsets::ALPHANUMERIC;
use random_string::generate;
use std::env;
use std::process::Command;

fn main() {
    set_version();
    set_repo_url();
}

fn set_repo_url() {
    let cargo_repo_url = env!("CARGO_PKG_REPOSITORY");
    let mut repo_url = cargo_repo_url.to_string();

    if is_git_repo() {
        repo_url = get_remote_url();
        if repo_url.starts_with("https://github.com/") {
            // The URL might already end with a '/', but GitHub seems to handle it fine if there's two.
            // Tested in both Firefox and Chromium.
            repo_url.push_str(&format!(
                "/tree/{}",
                if is_git_tagged() {
                    get_git_tag()
                } else {
                    get_git_hash()
                }
            ));
        }
    } else {
        println!("Not a Git repo! Skipping repo URL metadata");
    }

    println!("cargo:rustc-env=IN_REPO_URL={repo_url}");
}

fn set_version() {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let mut version = cargo_version.to_string();

    if is_git_repo() {
        let is_repo_dirty = is_repo_dirty();
        if !is_git_tagged() || is_repo_dirty {
            version.push_str("+rev.");
            version.push_str(&get_git_hash());

            // Add some randomness if dirty to avoid the browser caching resources while iterating.
            if is_repo_dirty {
                version.push_str("-dirty-");
                version.push_str(&generate(4, ALPHANUMERIC));
            }
        }
    } else {
        println!("Not a Git repo! Skipping version metadata");
    }

    println!("cargo:rustc-env=IN_VERSION={version}");
}

fn is_git_repo() -> bool {
    if let Ok(var) = env::var("IN_IS_GIT") {
        return var == "true";
    }
    Command::new("git").args(["status"]).status().is_ok()
}

fn get_git_hash() -> String {
    if let Ok(var) = env::var("IN_GIT_HASH") {
        return var;
    }

    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn is_repo_dirty() -> bool {
    if let Ok(var) = env::var("IN_GIT_DIRTY") {
        return var == "true";
    }

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("Failed to execute git status command");

    !status_output.stdout.is_empty()
}

fn is_git_tagged() -> bool {
    if let Ok(var) = env::var("IN_GIT_TAGGED") {
        return var == "true";
    }

    get_git_tag_info().0
}

fn get_git_tag() -> String {
    if let Ok(var) = env::var("IN_GIT_TAG") {
        return var;
    }

    get_git_tag_info().1
}

fn get_git_tag_info() -> (bool, String) {
    let output = Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .expect("Failed to execute git describe command");

    let is_tagged = output.status.success();
    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();

    (is_tagged, tag)
}

fn get_remote_url() -> String {
    if let Ok(var) = env::var("IN_GIT_REMOTE_URL") {
        return var;
    }

    let remote_url = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8_lossy(&remote_url.stdout)
        .trim()
        .to_string()
}
