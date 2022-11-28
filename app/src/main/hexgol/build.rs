// use std::env;
// use std::path::PathBuf;

fn main() {
    // // Tell cargo to invalidate the built crate whenever the wrapper changes
    // println!("cargo:rerun-if-changed=wrapper.h");

    // // The bindgen::Builder is the main entry point
    // // to bindgen, and lets you build up options for
    // // the resulting bindings.
    // let bindings = bindgen::Builder::default()
    //     // The input header we would like to generate
    //     // bindings for.
    //     .header("wrapper.h").
    //     // Tell cargo to invalidate the built crate whenever any of the
    //     // included header files changed.
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    //     // Finish the builder and generate the bindings.
    //     .generate()
    //     // Unwrap the Result and panic on failure.
    //     .expect("Unable to generate bindings");

    // // Write the bindings to the $OUT_DIR/bindings.rs file.
    // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    // bindings
    //     .write_to_file(out_path.join("bindings.rs"))
    //     .expect("Couldn't write bindings!");

    cc::Build::new()
        .cpp(true)
        .include("game-activity-csrc")
        .file("game-activity-csrc/game-activity/GameActivity.cpp")
        .extra_warnings(false)
        .cpp_link_stdlib("c++_static")
        .compile("game_activity");
    cc::Build::new()
        .cpp(true)
        .include("game-activity-csrc")
        .file("game-activity-csrc/game-text-input/gametextinput.cpp")
        .extra_warnings(false)
        .cpp_link_stdlib("c++_static")
        .compile("game_text_input");
    cc::Build::new()
        .include("game-activity-csrc")
        .include("game-activity-csrc/game-activity/native_app_glue")
        .file("game-activity-csrc/game-activity/native_app_glue/android_native_app_glue.c")
        .extra_warnings(false)
        .cpp_link_stdlib("c++_static")
        .compile("native_app_glue");

    // We need to link to both c++_static and c++abi for the static C++ library.
    // Ideally we'd link directly to libc++.a.
    println!("cargo:rustc-link-lib=c++abi");
    println!("cargo:rustc-link-lib=android");
}
