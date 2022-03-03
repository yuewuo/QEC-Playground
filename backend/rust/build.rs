extern crate cc;
use std::env;

fn main() {

    let target_os = env::var("CARGO_CFG_TARGET_OS");

    cc::Build::new()
        .cpp(true)
        .file("../../backend/blossomV/test.cpp")
        .compile("test");
    
    let mut build = cc::Build::new();

    build.cpp(true)
        .file("../../backend/blossomV/blossomV.cpp")
        .file("../../backend/blossomV/PMinterface.cpp")
        .file("../../backend/blossomV/PMduals.cpp")
        .file("../../backend/blossomV/PMexpand.cpp")
        .file("../../backend/blossomV/PMinit.cpp")
        .file("../../backend/blossomV/PMmain.cpp")
        .file("../../backend/blossomV/PMrepair.cpp")
        .file("../../backend/blossomV/PMshrink.cpp")
        .file("../../backend/blossomV/misc.cpp")
        .file("../../backend/blossomV/MinCost/MinCost.cpp");

    if target_os != Ok("macos".to_string()) {  // exclude from macOS
        build.cpp_link_stdlib("stdc++"); // use libstdc++
    }

    build.flag("-Wno-unused-parameter")
        .flag("-Wno-unused-variable")
        .flag("-Wno-unused-but-set-variable")
        .compile("blossomV");

    println!("cargo:rerun-if-changed=../../backend/blossomV/test.cpp");
    println!("cargo:rerun-if-changed=../../backend/blossomV/blossomV.cpp");

    if target_os != Ok("macos".to_string()) {  // exclude from macOS
        println!("cargo:rustc-link-lib=static=stdc++");  // have to add this to compile c++ (new, delete operators)
    }

    println!("cargo:rustc-link-lib=static=test");
    println!("cargo:rustc-link-lib=static=blossomV");
}
