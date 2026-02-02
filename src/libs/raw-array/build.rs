// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! Build script for raw-array crate.
//!
//! This sets up Verus verification configuration when the `verus` feature is enabled.

fn main() {
    // Register custom cfg values for Verus.
    println!("cargo::rustc-check-cfg=cfg(verus_keep_ghost)");
    println!("cargo::rustc-check-cfg=cfg(verus_keep_ghost_body)");

    // Set up Verus arguments when verifying.
    init_verify();

    println!("cargo:rerun-if-changed=build.rs");
}

fn init_verify() {
    // Check if we should skip verification (for faster iteration).
    if std::env::var("VERUS_NO_VERIFY").is_ok() {
        println!("cargo:rustc-env=VERUS_ARGS=--no-verify");
    } else {
        // Default Verus arguments for verification.
        let verus_args: [&str; 6] = [
            "--rlimit=4",
            "--expand-errors",
            "--multiple-errors=5",
            "--no-auto-recommends-check",
            "--trace",
            "-Z unstable-options",
        ];
        println!("cargo:rustc-env=VERUS_ARGS={}", verus_args.join(" "));
    }
}
