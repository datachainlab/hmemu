use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // If the package has a build script, this is set to the folder where the build script should place its output.
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("go")
        .args(&[
            "build",
            "-o",
            &format!("{}/libhm.so", out_dir),
            "-buildmode=c-shared",
            "./lib",
        ])
        .current_dir(&Path::new("."))
        .status()
        .unwrap();

    println!(r"cargo:rustc-link-lib=dylib=hm");
    println!(r"cargo:rustc-link-search=native={}", &out_dir);
}
