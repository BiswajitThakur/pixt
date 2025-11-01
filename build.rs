#![allow(unreachable_code)]

use std::{env, fs, path::Path};

fn main() {
    // Detect target triple dynamically
    let target = env::var("TARGET").unwrap();
    if !target.contains("wasm32") {
        return;
    }

    // Get version from Cargo.toml metadata
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    // Paths
    let src_path = Path::new("index.html");
    let dst_path = Path::new("pkg/index.html");

    // Read HTML
    let html = fs::read_to_string(src_path).expect("Failed to read index.html");

    // Inject version — look for a marker like {{VERSION}} in your HTML
    let new_html = html.replace("{{VERSION}}", &format!("v{}", version));

    // Create pkg folder if missing
    if let Some(parent) = dst_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    // Write to pkg/index.html
    fs::write(dst_path, new_html).expect("Failed to write pkg/index.html");

    println!(
        "cargo:warning=Generated pkg/index.html with version {}",
        version
    );
}
