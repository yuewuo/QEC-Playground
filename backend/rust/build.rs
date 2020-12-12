extern crate cc;

fn main() {

    cc::Build::new()
        .file("../../backend/blossomV/test.c")
        .compile("test");
    
    cc::Build::new()
        .file("../../backend/blossomV/blossomV.cpp")
        .compile("blossomV");

    println!("cargo:rerun-if-changed=../../backend/blossomV/test.c");
    println!("cargo:rerun-if-changed=../../backend/blossomV/blossomV.cpp");
    println!("cargo:rustc-link-lib=static=test");
    println!("cargo:rustc-link-lib=static=blossomV");
}
