use std::path::PathBuf;

fn main() {
    // Collect all .proto files under proto/**/*
    let mut protos: Vec<PathBuf> = Vec::new();
    for entry in glob::glob("proto/*.proto").expect("Failed to read glob pattern") {
        let path = entry.expect("Invalid path from glob");
        // Emit rerun-if-changed for each file so Cargo rebuilds if you edit proto
        println!("cargo:rerun-if-changed={}", path.display());
        protos.push(path);
    }

    // Also rerun if the include root changes
    println!("cargo:rerun-if-changed=proto");

    // Include roots (add more if you import from other dirs)
    let includes = &["proto"];

    // Configure prost-build (tweak as you like)
    let mut cfg = prost_build::Config::new();

    // Example: map well-known types to prost-types (usually automatic, shown for clarity)
    // cfg.extern_path(".google.protobuf", "::prost_types");

    // Example: Keep bytes as Vec<u8> for everything (or restrict with matching paths)
    // cfg.bytes(&["."]);

    cfg.compile_protos(
        &protos.iter().map(|p| p.as_path()).collect::<Vec<_>>(),
        includes,
    ).expect("prost-build failed");
}
