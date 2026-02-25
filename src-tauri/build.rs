use std::process::Command;

fn main() {
    // Get short git commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git");

    let git_hash = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .to_string();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    tauri_build::build()
}
