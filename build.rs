use random_string::charsets::ALPHANUMERIC;
use random_string::generate;
use std::env;
use std::process::Command;

fn main() {
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

    println!("cargo:rustc-env=IN_VERSION={}", version);
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

    String::from_utf8(output.stdout).unwrap().trim().to_string()
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

    let output = Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .expect("Failed to execute git describe command");

    output.status.success()
}
