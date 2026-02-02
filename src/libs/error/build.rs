// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

fn main() {
    // Register Verus cfg flags to prevent warnings.
    println!("cargo::rustc-check-cfg=cfg(verus_keep_ghost)");
    println!("cargo::rustc-check-cfg=cfg(verus_keep_ghost_body)");
}
