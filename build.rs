use std::env;
use std::fs;

fn main() {
    let embedded_binary = env::var("EMBEDDED_BINARY").expect("EMBEDDED_BINARY not set");
    if !fs::metadata(&embedded_binary).is_ok() {
        panic!("Binary file '{}' not found", embedded_binary);
    }

    println!("cargo:rustc-env=EMBEDDED_BINARY={}", embedded_binary);
}
