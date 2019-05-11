fn main() {
    println!(r"cargo:rustc-link-lib=dylib=hm");
    println!(r"cargo:rustc-link-search=native=./build");
}
