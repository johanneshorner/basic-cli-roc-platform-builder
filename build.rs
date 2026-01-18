use std::{path::Path, process::Command};

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if !target.ends_with("-musl") {
        return;
    }

    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("rustc must be available");

    if !output.status.success() {
        panic!("rustc sysroot command failed: {:?}", output);
    }

    let sysroot: &Path = str::from_utf8(output.stdout.trim_ascii())
        .expect("valid UTF-8 from rustc")
        .as_ref();

    let musl_lib_path = sysroot
        .join("lib")
        .join("rustlib")
        .join(&target)
        .join("lib/self-contained");

    println!(
        "cargo::rustc-link-search=native={}",
        musl_lib_path.to_str().unwrap()
    );
    println!("cargo::rustc-link-lib=static=unwind");
}
