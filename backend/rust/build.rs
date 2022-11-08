extern crate cc;
use std::env;
use std::path::Path;

fn main() {

    if Path::new("../blossomV/PerfectMatching.h").exists() {

        println!("cargo:rustc-cfg=feature=\"blossom_v\"");

        let target_os = env::var("CARGO_CFG_TARGET_OS");
        
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
            build.flag("-Wno-unused-but-set-variable");  // this option is not available in clang
        }

        build.flag("-Wno-unused-parameter")
            .flag("-Wno-unused-variable")
            .flag("-Wno-reorder-ctor")
            .flag("-Wno-reorder")
            .compile("blossomV");

        println!("cargo:rerun-if-changed=../../backend/blossomV/blossomV.cpp");
        println!("cargo:rerun-if-changed=../../backend/blossomV/PerfectMatching.h");

        println!("cargo:rustc-link-lib=static=blossomV");

        if target_os != Ok("macos".to_string()) {  // exclude from macOS
            // println!("cargo:rustc-link-lib=static=stdc++");  // have to add this to compile c++ (new, delete operators)
            println!("cargo:rustc-link-lib=dylib=stdc++");  // NOTE: this MUST be put after "cargo:rustc-link-lib=static=blossomV"
        }

    }
}
