extern crate cc;

fn main() {

    cc::Build::new()
        .file("../../backend/blossomV/test.c")
        .compile("test");
    
    cc::Build::new()
        .cpp(true)
        .file("../../backend/blossomV/blossomV.cpp")
        .file("../../backend/blossomV/PMinterface.cpp")
        .file("../../backend/blossomV/PMduals.cpp")
        .file("../../backend/blossomV/PMexpand.cpp")
        .file("../../backend/blossomV/PMinit.cpp")
        .file("../../backend/blossomV/PMmain.cpp")
        .file("../../backend/blossomV/PMrepair.cpp")
        .file("../../backend/blossomV/PMshrink.cpp")
        .file("../../backend/blossomV/misc.cpp")
        .file("../../backend/blossomV/MinCost/MinCost.cpp")
        .cpp_link_stdlib("stdc++") // use libstdc++
        .flag("-Wreorder")
        .compile("blossomV");

    println!("cargo:rerun-if-changed=../../backend/blossomV/test.c");
    println!("cargo:rerun-if-changed=../../backend/blossomV/blossomV.cpp");
    println!("cargo:rustc-link-lib=static=stdc++");  // have to add this to compile c++ (new, delete operators)
    println!("cargo:rustc-link-lib=static=test");
    println!("cargo:rustc-link-lib=static=blossomV");
}
