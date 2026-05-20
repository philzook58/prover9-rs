use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ladr_dir = manifest_dir.join("../Prover9/ladr");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/shim.c");
    println!("cargo:rerun-if-changed={}", ladr_dir.display());

    let mut sources = ladr_sources(&ladr_dir);
    sources.push(manifest_dir.join("src/shim.c"));

    let mut build = cc::Build::new();
    build
        .flag(format!("-iquote{}", ladr_dir.display()))
        .warnings(false);
    for source in &sources {
        build.file(source);
    }
    build.compile("prover9");
}

fn ladr_sources(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("c") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("test.c") {
            continue;
        }
        files.push(path);
    }
    files.sort();
    files
}
