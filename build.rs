fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let lib_path = std::path::PathBuf::from("lib51degrees");
    let src_dir = lib_path.join("src");

    // cmake build
    let dst = cmake::Config::new(&lib_path)
        .define("MemoryOnly", "YES")
        .define("BUILD_TESTING", "OFF")
        .profile("Release")
        .build();

    let built_lib_dir = dst.join("build").join("lib");

    println!("cargo:rerun-if-changed={}", lib_path.display());
    println!("cargo:rustc-link-search=native={}", built_lib_dir.display());
    println!("cargo:rustc-link-lib=static=fiftyone-hash-c");
    println!("cargo:rustc-link-lib=static=fiftyone-device-detection-c");
    println!("cargo:rustc-link-lib=static=fiftyone-common-c");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=atomic");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=/usr/local/lib");
        println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    }

    // generate bindings

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", src_dir.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate_inline_functions(true)
        .allowlist_type("fiftyoneDegrees.*")
        //.allowlist_type("fiftyone_degrees_string.*")
        .allowlist_var("fiftyoneDegrees.*")
        .allowlist_function("fiftyoneDegrees.*")
        //.allowlist_function("fiftyone_degrees_.*")
        //.allowlist_function("FIFTYONE_DEGREES_.*")
        //.allowlist_function("fiftyonedegrees.*")
        .layout_tests(false)
        .derive_default(true)
        .derive_copy(true)
        .derive_debug(true);

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .clang_arg("-I/usr/local/include")
            .clang_arg("-I/opt/homebrew/include");
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");

    Ok(())
}
