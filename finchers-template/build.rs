use std::env;

fn main() {
    if env::var_os("FINCHERS_DENY_WARNINGS").is_some() {
        println!("cargo:rustc-cfg=finchers_deny_warnings");
    }
}
