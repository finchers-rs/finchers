extern crate version_check;

use std::env;

fn main() {
    if env::var_os("FINCHERS_DENY_WARNINGS").is_some() {
        println!("cargo:rustc-cfg=finchers_deny_warnings");
    }

    match version_check::is_min_version("1.30.0") {
        Some((true, _ver)) => println!("cargo:rustc-cfg=use_external_macros"),
        _ => {}
    }
}
